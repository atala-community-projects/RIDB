[**@elribonazo/ridb**](../README.md) • **Docs**

***

[@elribonazo/ridb](../README.md) / RIDB

# Class: RIDB\<T\>

## Type Parameters

• **T** *extends* [`SchemaTypeRecord`](../namespaces/RIDBTypes/type-aliases/SchemaTypeRecord.md)

## Constructors

### new RIDB()

> **new RIDB**\<`T`\>(`schemas`): [`RIDB`](RIDB.md)\<`T`\>

#### Parameters

• **schemas**: `T`

#### Returns

[`RIDB`](RIDB.md)\<`T`\>

#### Defined in

[ts/src/index.ts:171](https://github.com/elribonazo/RIDB/blob/3648b42af4cf7bbbd1da0177549767bef8e1fefc/ts/src/index.ts#L171)

## Accessors

### collections

> `get` **collections**(): \{ \[name in string \| number \| symbol\]: Collection\<Schema\<T\[name\]\>\> \}

#### Returns

\{ \[name in string \| number \| symbol\]: Collection\<Schema\<T\[name\]\>\> \}

#### Defined in

[ts/src/index.ts:182](https://github.com/elribonazo/RIDB/blob/3648b42af4cf7bbbd1da0177549767bef8e1fefc/ts/src/index.ts#L182)

## Methods

### start()

> **start**(`storageType`?): `Promise`\<[`Database`](../namespaces/RIDBTypes/classes/Database.md)\<`T`\>\>

#### Parameters

• **storageType?**: *typeof* [`BaseStorage`](../namespaces/RIDBTypes/classes/BaseStorage.md)

#### Returns

`Promise`\<[`Database`](../namespaces/RIDBTypes/classes/Database.md)\<`T`\>\>

#### Defined in

[ts/src/index.ts:195](https://github.com/elribonazo/RIDB/blob/3648b42af4cf7bbbd1da0177549767bef8e1fefc/ts/src/index.ts#L195)
