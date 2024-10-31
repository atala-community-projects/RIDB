/// <reference types="@hyperledger/identus-edge-agent-sdk" />

const {
    RIDB,
} = require('../build/cjs/index.cjs');

const SDK = require("@hyperledger/identus-edge-agent-sdk");

const mediatorDID = SDK.Domain.DID.fromString(
    "did:peer:2.Ez6LSghwSE437wnDE1pt3X6hVDUQzSjsHzinpX3XFvMjRAm7y.Vz6Mkhh1e5CEYYq6JBUcTZ6Cp2ranCWRrv7Yax3Le4N59R6dd.SeyJ0IjoiZG0iLCJzIjp7InVyaSI6Imh0dHA6Ly8xOTIuMTY4LjEuNDQ6ODA4MCIsImEiOlsiZGlkY29tbS92MiJdfX0.SeyJ0IjoiZG0iLCJzIjp7InVyaSI6IndzOi8vMTkyLjE2OC4xLjQ0OjgwODAvd3MiLCJhIjpbImRpZGNvbW0vdjIiXX19"
);

(async () => {

    class RIDBStore  {
        _db = null

        get collections() {
            return this._db.collections
        }

        async start() {
            const db = new RIDB(
                {
                    'credentials': SDK.Models.CredentialSchema,
                    'credentialMetadata': SDK.Models.CredentialMetadataSchema,
                    'didkeyLink': SDK.Models.DIDKeyLinkSchema,
                    'didLink': SDK.Models.DIDLinkSchema,
                    'dids': SDK.Models.DIDSchema,
                    'keys': SDK.Models.KeySchema,
                    'messages': SDK.Models.MessageSchema
                }
            )
            await db.start()
            this._db = db;
        }

        async update(name, model) {
            const collection = this.collections[name]
            await collection.update(model)
        }

        async delete(name, uuid) {
            const collection = this.collections[name]
            await collection.remove(uuid)
        }

        async query(name, query) {
            const collection = this.collections[name]
            const queryResponse = await collection.find(query.selector)
            return queryResponse
        }

        async insert(name, data) {
            const collection = this.collections[name]
            const queryResponse = await collection.create(data);
            return queryResponse
        }

        async cleanup() {
            throw new Error("Not implemented")
        }

        async clear() {
            throw new Error("Not implemented")
        }
    }

    const apollo = new SDK.Apollo();
    const castor = new SDK.Castor(apollo);
    const pluto =  new SDK.Pluto(
        new RIDBStore(),
        apollo
    );
    const mediationStore = new SDK.PublicMediatorStore(pluto);
    const api = new SDK.ApiImpl();
    const seed = apollo.createRandomSeed().seed;
    const didcomm = new SDK.DIDCommWrapper(apollo, castor, pluto);
    const mercury = new SDK.Mercury(castor, didcomm, api);
    const pollux = new SDK.Pollux(apollo, castor);

    await pollux.start();

    await pollux.anoncreds.createLinksecret()

    const handler = new SDK.BasicMediatorHandler(mediatorDID, mercury, mediationStore);
    const manager = new SDK.ConnectionsManager(
        castor, 
        mercury,
        pluto,
        pollux,
        handler
    );
    
    const agent = new SDK.Agent(
        apollo,
        castor,
        pluto, 
        mercury,
        handler,
        manager,
        seed
    );

    console.log("Starting the database")

    agent.addListener(SDK.ListenerKey.MESSAGE, (messages) => {

        console.log(messages);
        debugger
    })

    await agent.start()
    console.log("Ok :)")

})()
