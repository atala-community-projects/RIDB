import { describe, it, expect } from 'vitest';
import { SchemaFieldType, RIDB, type RIDBTypes } from '..';

const storages: (typeof RIDBTypes.BaseStorage<RIDBTypes.SchemaType>|undefined)[] = [
    undefined
]

export default (platform: string) =>

describe(`[${platform}] Testing`, () => {

    storages.forEach(storage => {
        describe(`[${platform}][${storage ? 'Typescript' : 'Wasm'}] Testing Storage`, () => {
            it('It should be able to create a new document from JSON schema', async () => {

                const db =  new RIDB(
                    {
                        demo: {
                            version: 0,
                            primaryKey: 'id',
                            type:SchemaFieldType.object,
                            properties: {
                                id: {
                                    type:SchemaFieldType.string,
                                    maxLength: 60
                                }
                            }
                        }
                    }
                )
                await db.start(storage);
                expect(db).to.not.be.undefined;
                expect(db.collections).to.not.be.undefined;
                expect(db.collections).to.haveOwnProperty("demo");
                expect(db.collections.demo).to.not.be.undefined;
                expect(db.collections.demo.find).to.not.be.undefined;

                const created = await db.collections.demo.create({
                    id: "12345"
                })
                
                expect(created).to.not.be.undefined;
                expect(created).to.haveOwnProperty("id");
                expect(created.id).to.eq("12345")
            })
            it("should allow optional fields", async () => {
                const db =  new RIDB(
                    {
                        demo: {
                            version: 0,
                            primaryKey: 'id',
                            type:SchemaFieldType.object,
                            properties: {
                                id: {
                                    type:SchemaFieldType.string,
                                    maxLength: 60
                                },
                                name: {
                                    type: SchemaFieldType.string,
                                    maxLength: 20,
                                    required:false
                                }
                            }
                        }
                    }
                )
                
                await db.start(storage);

                expect(db).to.not.be.undefined;
                expect(db.collections).to.not.be.undefined;
                expect(db.collections).to.haveOwnProperty("demo");
                expect(db.collections.demo).to.not.be.undefined;
                expect(db.collections.demo.find).to.not.be.undefined;

                const created = await db.collections.demo.create({
                    id: "12345",
                });
                expect(created).to.not.be.undefined;
                expect(created).to.haveOwnProperty("id");
                expect(created).to.not.haveOwnProperty("name");

                expect(created.id).to.eq("12345")

            })
            it("Should allow updating a document", async () => {
                const db =  new RIDB(
                    {
                        demo: {
                            version: 0,
                            primaryKey: 'id',
                            type:SchemaFieldType.object,
                            properties: {
                                id: {
                                    type:SchemaFieldType.string,
                                    maxLength: 60
                                },
                                name: {
                                    type: SchemaFieldType.string,
                                    maxLength: 20
                                },
                            }
                        }
                    }
                )

                await db.start(storage);

                expect(db).to.not.be.undefined;
                expect(db.collections).to.not.be.undefined;
                expect(db.collections).to.haveOwnProperty("demo");
                expect(db.collections.demo).to.not.be.undefined;
                expect(db.collections.demo.find).to.not.be.undefined;

                const created = await db.collections.demo.create({
                    id: "12345",
                    name: "demo"
                })
                expect(created).to.not.be.undefined;
                expect(created).to.haveOwnProperty("id");
                expect(created).to.haveOwnProperty("name");

                expect(created.id).to.eq("12345")
                expect(created.name).to.eq("demo")

                await db.collections.demo.update({
                    ...created,
                    name: "demo2"
                })

                const result = await db.collections.demo.findById(created.id);
                expect(result).to.not.be.undefined;
                expect(result).to.haveOwnProperty("id");
                expect(result).to.haveOwnProperty("name");
                expect(result.id).to.eq("12345")
                expect(result.name).to.eq("demo2")
            

            })
            it("Should be able to create a default database with a valid schema", async () => {
                const db =  new RIDB(
                    {
                        demo: {
                            version: 0,
                            primaryKey: 'id',
                            type:SchemaFieldType.object,
                            properties: {
                                id: {
                                    type:SchemaFieldType.string,
                                    maxLength: 60
                                }
                            }
                        }
                    }
                )
                await db.start(storage)
                expect(db).to.not.be.undefined;
            });
            it("Should be able to find a created schema entry", async () => {
                const db =  new RIDB(
                    {
                        demo: {
                            version: 0,
                            primaryKey: 'id',
                            type:SchemaFieldType.object,
                            properties: {
                                id: {
                                    type:SchemaFieldType.string,
                                    maxLength: 60
                                },
                                age: {
                                    type:SchemaFieldType.number,
                                }
                            }
                        }
                    }
                )
                await db.start(storage)
                expect(db).to.not.be.undefined;

                const created = await db.collections.demo.create({
                    id: "12345",
                    age: 18
                })
                expect(created).to.not.be.undefined;
                expect(created).to.haveOwnProperty("id");
                expect(created).to.haveOwnProperty("age");

                const found = await db.collections.demo.find({
                    age: {
                        $gte:2
                    },
                    $or: [
                        {
                            age: {
                                $in:[2,3,4, 18, 19, 20],
                                $gte:2
                            }
                        }
                    ]
                })

                expect(found.length).to.eq(1)

            });
            it("Should be able to count a created schema entry", async () => {
                const db =  new RIDB(
                    {
                        demo: {
                            version: 0,
                            primaryKey: 'id',
                            type:SchemaFieldType.object,
                            properties: {
                                id: {
                                    type:SchemaFieldType.string,
                                    maxLength: 60
                                },
                                age: {
                                    type:SchemaFieldType.number,
                                }
                            }
                        }
                    }
                )
                await db.start(storage)
                expect(db).to.not.be.undefined;

                const created = await db.collections.demo.create({
                    id: "12345",
                    age: 18
                })
                expect(created).to.not.be.undefined;
                expect(created).to.haveOwnProperty("id");
                expect(created).to.haveOwnProperty("age");

                const found = await db.collections.demo.count({
                    age: {
                        $gte:2
                    },
                    $or: [
                        {
                            age: {
                                $in:[2,3,4, 18, 19, 20],
                                $gte:2
                            }
                        }
                    ]
                })

                expect(found).to.eq(1)

            });
            it("Should throw an error with a schema with invalid type", async () => {
                const db =  new RIDB(
                    {
                        demo: {
                            version: 0,
                            primaryKey: 'id',
                            type:"wrong",
                            properties: { }
                        }
                    }
                )
                await expect(async () => db.start(storage)).to.rejects.toThrowError("Validation Error: Schema type is invalid (\"wrong\")")
            })
            it("Should throw an error when schema properties type is invalid", async () => {
                const db =  new RIDB(
                    {
                        demo: {
                            version: 0,
                            primaryKey: 'id',
                            type:"obiect",
                            properties: {
                                id: {
                                    type:"....",
                                    minLength:-1
                                }
                            }
                        }
                    }
                )
                await expect(async () => db.start(storage)).to.rejects.toThrowError("Serialization Error: invalid value: string \"....\", expected an PropertyType (String, Number, Boolean, Object or Array)")
            })
            it("Should throw an error when the minLength is lower than 0", async () => {
                const db =  new RIDB(
                    {
                        demo: {
                            version: 0,
                            primaryKey: 'id',
                            type:"object",
                            properties: {
                                id: {
                                    type:"string",
                                    minLength:-1
                                }
                            }
                        }
                    }
                )
                await expect(async () => db.start(storage)).to.rejects.toThrowError("Validation Error: Min property not valid")
            })
            it("Should throw an error when schemaType with a property that has min higher than max", async () => {
                const db =  new RIDB(
                    {
                        demo: {
                            version: 0,
                            primaryKey: 'id',
                            type:"object",
                            properties: {
                                id: {
                                    type:"string",
                                    maxLength: 2,
                                    minLength:3
                                }
                            }
                        }
                    }
                )
                await expect(async () => db.start(storage)).to.rejects.toThrowError("Validation Error: Min higher than max")
            });
        })
    })
})