[**@trust0/ridb**](../README.md) • **Docs**

***

[@trust0/ridb](../README.md) / RIDB

# Class: RIDB\<T\>

Represents a RIDB (Rust IndexedDB) instance.

## Type Parameters

• **T** *extends* [`SchemaTypeRecord`](../namespaces/RIDBTypes/type-aliases/SchemaTypeRecord.md)

The type of the schema record.

## Constructors

### new RIDB()

> **new RIDB**\<`T`\>(`options`): [`RIDB`](RIDB.md)\<`T`\>

Creates an instance of RIDB.

#### Parameters

• **options**: `object` & `MigrationsParameter`\<`T`\>

#### Returns

[`RIDB`](RIDB.md)\<`T`\>

#### Defined in

[ts/src/index.ts:70](https://github.com/elribonazo/RIDB/blob/1c63aad9f4ff19e9fbbdb4c1cc728eac5263e72c/ts/src/index.ts#L70)

## Accessors

### collections

#### Get Signature

> **get** **collections**(): \{ \[name in string \| number \| symbol\]: Collection\<Schema\<T\[name\]\>\> \}

Gets the collections from the database.

##### Returns

\{ \[name in string \| number \| symbol\]: Collection\<Schema\<T\[name\]\>\> \}

The collections object.

#### Defined in

[ts/src/index.ts:109](https://github.com/elribonazo/RIDB/blob/1c63aad9f4ff19e9fbbdb4c1cc728eac5263e72c/ts/src/index.ts#L109)

## Methods

### start()

> **start**(`options`?): `Promise`\<[`Database`](../namespaces/RIDBTypes/classes/Database.md)\<`T`\>\>

Starts the database.

#### Parameters

• **options?**

• **options.password?**: `string`

• **options.storageType?**: *typeof* [`BaseStorage`](../namespaces/RIDBTypes/classes/BaseStorage.md)

#### Returns

`Promise`\<[`Database`](../namespaces/RIDBTypes/classes/Database.md)\<`T`\>\>

A promise that resolves to the database instance.

#### Defined in

[ts/src/index.ts:150](https://github.com/elribonazo/RIDB/blob/1c63aad9f4ff19e9fbbdb4c1cc728eac5263e72c/ts/src/index.ts#L150)