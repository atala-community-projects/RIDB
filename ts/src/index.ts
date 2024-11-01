/**
 * @packageDocumentation
 *
 * RIDB is a storage agnostic secure database wrapper for the web, written in rust.
 * The project started after years of experience working with web projects in both browser and nodejs platforms, the project was born with some rules / objectives:
 * 1. Strong types + proper validation
 * 2. Declarative schemas & documents
 * 3. Configurable storages, inMemory, monogoDB, sqlite, indexdb
 * 4. Secure encryption
 * 5. Work seamlessly in browsers or nodejs applications.
 *
 * ## Supported features for InMemory Storage
 * The inMemory storage is used by default and is currently supporting the following features:
 * 1. Schemas: Creation of declararive schemas with required fields
 * 2. Schemas: Implement validation across all the flows extracting properties and required fields when needed
 * 3. Schemas: Manage Primary keys
 * 4. Internal Storage: write operation, create, update, fetch one, remove, find and count
 * 5. Internal Storage: Rust inMemory implementation
 * 6. Database default InMemory plugged in
 *
 * ## Install
 * 
 * npm: 
 * ``` 
 * npm i @elribonazo/ridb --save
 * ```
 * 
 * yarn:
 * 
 * ``` 
 * yarn add @elribonazo/ridb
 * ```
 * 
 * ## Usage
 * 
 * ### Database
 * 
 * In CommonJS Modules:  
 * 
 * ```javascript
 * const {
 *     RIDB,
 *     SchemaFieldType
 * } = require('@elribonazo/ridb');
 * 
 * (async () => {
 *     const db =  new RIDB({
 *         demo: {
 *             version: 0,
 *             primaryKey: 'id',
 *             type: SchemaFieldType.object,
 *             properties: {
 *                 id: {
 *                     type: SchemaFieldType.string,
 *                     maxLength: 60
 *                 }
 *             }
 *         }
 *     });
 *     console.log("Starting the database");
 *     await db.start();
 *     console.log("Ok :)");
 * })()
 * ```
 * 
 * In ES Modules, TypeScript, etc:
 * 
 * ```javascript
 * import {
 *     RIDB,
 *     SchemaFieldType
 * } from '@elribonazo/ridb';
 * 
 * (async () => {
 *     const db =  new RIDB({
 *         demo: {
 *             version: 0,
 *             primaryKey: 'id',
 *             type: SchemaFieldType.object,
 *             properties: {
 *                 id: {
 *                     type: SchemaFieldType.string,
 *                     maxLength: 60
 *                 }
 *             }
 *         }
 *     });
 *     console.log("Starting the database");
 *     await db.start();
 *     console.log("Ok :)");
 * })()
 * ```
 * 
 * ## How to run tests
 * 
 * ```bash
 * cd ts 
 * npm i
 * npm run test
 * ```
 * 
 * ## How to build this project
 * Build requirements:
 * * Bash
 * * Have Rust ([cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)) and [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)) installed.
 * * Node JS Version (20/LTS Recommended)
 * 
 * ```bash
 * cd ts 
 * npm i
 * npm run build
 * ```
 * 
 * ## How to test the project
 * For now, we have enabled the implementation of the whole wasm + javascript integration.
 * In order to run it, write the following:
 * 
 * ```bash
 * cd ts 
 * npm i
 * npm run test
 * ```
 *
 * ## Create your own storage
 * A valid storage must extend [BaseStorage class](https://github.com/atala-community-projects/RIDB/blob/main/namespaces/RIDBTypes/classes/BaseStorage.md)
 * here's some example:
 *
 * ```typescript
 * export class InMemory<T extends SchemaType> extends BaseStorage<T>   {
 *     async write(operation:Operation<T>): Promise<Doc<T>> {
 *         if (operation.opType === OpType.CREATE) {
 *             return operation.data;
 *         }
 *         throw new Error("Method not implemented.");
 *     }
 *
 *     query(): Promise<void> {
 *         throw new Error("Method not implemented.");
 *     }
 *
 *     findDocumentById(id: string): Promise<null> {
 *         throw new Error("Method not implemented.");
 *     }
 *
 *     count(): Promise<number> {
 *         throw new Error("Method not implemented.");
 *     }
 *
 *     close(): Promise<void> {
 *         throw new Error("Method not implemented.");
 *     }
 *
 * }
 * ```
 *
 *
 */
import wasmBuffer from "../../pkg/ridb_rust_bg.wasm";

import  * as RIDBTypes from "ridb-rust";
export  * as RIDBTypes from "ridb-rust";

export {BaseStorage } from 'ridb-rust';

export class RIDB<T extends RIDBTypes.SchemaTypeRecord> {
    private schemas: T;

    private _internal: typeof import("ridb-rust") | undefined;
    private _db: RIDBTypes.Database<T> | undefined;

    constructor(schemas: T) {
        this.schemas = schemas;
    }

    private get db() {
        if (!this._db) {
            throw new Error("Start the database first");
        }
        return this._db;
    }

    get collections() {
        return this.db.collections;
    }

    private async load(): Promise<typeof import("ridb-rust")> {
        this._internal ??= await import("ridb-rust").then(async module => {
            const wasmInstance = module.initSync(wasmBuffer);
            await module.default(wasmInstance);
            return module;
        });
        return this._internal!;
    }

    async start(storageType?: typeof RIDBTypes.BaseStorage<RIDBTypes.SchemaType>) {
        const {Database} = await this.load();
        this._db ??= await Database.create(this.schemas, {
            createStorage: (schemas) => this.createStorage(schemas, storageType)
        });
        return this.db;
    }

    private createStorage<J extends RIDBTypes.SchemaTypeRecord>(schemas: J, storageConstructor?: typeof RIDBTypes.BaseStorage<RIDBTypes.SchemaType>) {
        if (!this._internal) {
            throw new Error("Start the database first");
        }
        const Storage = storageConstructor ?? this._internal.InMemory;
        return Object.keys(schemas).reduce((storages, name) => ({
            ...storages,
            [name]: new Storage(name, schemas[name])
        }), {} as RIDBTypes.InternalsRecord);
    }
}

export const SchemaFieldType = {
    "string": 'string' as const,
    "number": 'number' as const,
    "boolean": 'boolean' as const,
    "array": 'array' as const,
    "object": 'object' as const,
};

