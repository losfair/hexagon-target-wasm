const fs = require("fs");
const path = require("path");

class Engine {
    constructor() {
        this.core = null;
        this.executor = null;
        this.enginePath = "engine.wasm";
        this.memory = null;
        this.programs = {};
        this.nextProgramId = 0;
    }

    async init() {
        this.core = await WebAssembly.instantiate(
            fs.readFileSync(this.enginePath),
            {
                env: {
                    hexagon_external_global_invoke_callback: this.invokeCallback.bind(this),
                    fmod: (a, b) => a % b
                }
            }
        );
        this.memory = this.core.instance.exports.memory;
        this.executor = this.core.instance.exports.hexagon_executor_create();
    }

    destroy() {
        this.core.instance.exports.hexagon_executor_destroy(this.executor);
        this.core = null;
        this.memory = null;
        this.executor = null;
        this.aotCache = {};
    }

    loadProgram(code, aotLoader) {
        let codeBuf = new Uint8Array(code);
        let codeChunk = this.alloc(codeBuf.length);

        this.writePtr(codeChunk, codeBuf, codeBuf.length);

        let program = this.core.instance.exports.hexagon_executor_load_program(
            this.executor,
            codeChunk,
            codeBuf.length,
            0,
            0
        );

        this.free(codeChunk);

        if(!program) {
            throw new Error("Unable to load program");
        }

        let programId = this.nextProgramId++;
        let inst = new ProgramInstance(this, programId, program, aotLoader);
        this.programs[programId] = inst;

        return inst;
    }

    invokeCallback(handle, fnId, userData) {
        let target = this.programs[userData];
        return target.invokeCallback(fnId);
    }

    alloc(len) {
        return this.core.instance.exports.hexagon_glue_alloc(len);
    }

    free(ptr) {
        this.core.instance.exports.hexagon_glue_free(ptr);
    }

    writePtr(ptr, data, len) {
        let arrayView = new Uint8Array(this.memory.buffer);
        for(let i = 0; i < len; i++) {
            arrayView[ptr + i] = data[i];
        }
    }

    writePtrByte(ptr, byte) {
        let arrayView = new Uint8Array(this.memory.buffer);
        arrayView[ptr] = byte;
    }

    readPtr(ptr, len) {
        let arrayView = new Uint8Array(this.memory.buffer);
        let result = new Uint8Array(len);
        for(let i = 0; i < len; i++) {
            result[i] = arrayView[ptr + i];
        }
        return result;
    }
}

class ProgramInstance {
    constructor(engine, id, context, aotLoader) {
        this.engine = engine;
        this.id = id;
        this.context = context;
        this.aotCache = {};
        this.aotLoader = aotLoader;
    }

    destroy() {
        this.engine.core.instance.exports.hexagon_context_destroy(this.context);
        delete this.engine.programs[this.id];
        this.engine = null;
        this.id = null;
        this.context = null;
        this.aotCache = null;
    }

    invokeCallback(fnId) {
        if(typeof(this.aotCache[fnId]) === "undefined") {
            // prevent recompilation
            this.aotCache[fnId] = null;
            this.compileToAotCache(fnId).catch(e => {
                console.log(e);
                if(this.aotCache) this.aotCache[fnId] = null;
            });
            return 0;
        }

        if(this.aotCache[fnId] === null) {
            return 0;
        } else {
            let m = this.aotCache[fnId];
            console.log(m.instance.exports);
            console.log("AOT EXECUTION BEGIN");
            m.instance.exports.main();
            console.log("AOT EXECUTION END");
            return 1;
        }
    }

    run() {
        this.engine.core.instance.exports.hexagon_context_run(this.context);
    }

    getGlobal(id) {
        console.log("getGlobal " + id);
        return this.engine.core.instance.exports.hexagon_context_get_global(this.context, id);
    }

    setGlobal(id, value) {
        console.log("setGlobal " + id + " " + value);
        return this.engine.core.instance.exports.hexagon_context_set_global(this.context, id, value);
    }

    async compileToAotCache(fnId) {
        let wasmData = await this.aotLoader(fnId);
        if(!wasmData) throw new Error("Cannot get wasm data");

        let targetModule = await WebAssembly.instantiate(wasmData, {
            env: {
                load_global: this.getGlobal.bind(this),
                store_global: this.setGlobal.bind(this)
            }
        });
        this.aotCache[fnId] = targetModule;
    }
}

async function run() {
    const binPrefix = process.argv[2];
    const masterModule = fs.readFileSync(
        path.join(binPrefix, "module.cfg")
    );

    let engine = new Engine();
    await engine.init();
    let program = engine.loadProgram(masterModule, fnId => {
        return fs.readFileSync(path.join(binPrefix, "" + fnId + ".wasm"));
    });
    await program.compileToAotCache(0);
    program.run();
    console.log(program.getGlobal(0));

    program.destroy();
    engine.destroy();
}

run().catch(e => console.log(e));
