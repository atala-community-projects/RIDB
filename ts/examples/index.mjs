/// <reference types="@hyperledger/identus-edge-agent-sdk" />

/** @type {SDK} */
import SDK from '@hyperledger/identus-edge-agent-sdk'

import {RIDB} from '../build/esm/index.mjs';

const mediatorDID = "did:peer:2.Ez6LSghwSE437wnDE1pt3X6hVDUQzSjsHzinpX3XFvMjRAm7y.Vz6Mkhh1e5CEYYq6JBUcTZ6Cp2ranCWRrv7Yax3Le4N59R6dd.SeyJ0IjoiZG0iLCJzIjp7InVyaSI6Imh0dHA6Ly8xOTIuMTY4LjEuNDQ6ODA4MCIsImEiOlsiZGlkY29tbS92MiJdfX0.SeyJ0IjoiZG0iLCJzIjp7InVyaSI6IndzOi8vMTkyLjE2OC4xLjQ0OjgwODAvd3MiLCJhIjpbImRpZGNvbW0vdjIiXX19";

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

        async query(name, query = {}) {
            const collection = this.collections[name]
            return collection.find(query?.selector || query)
        }

        async insert(name, data) {
            const collection = this.collections[name]
            return collection.create(data)
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
    const seed = apollo.createRandomSeed().seed;
    const agent =  SDK.Agent.initialize(
        {
            mediatorDID,
            apollo,
            castor,
            pluto,
            seed,
        }
    );
    agent.addListener(SDK.ListenerKey.MESSAGE, async (messages) => {
        console.log(messages);
        agent.stop()
    })
    console.log("Starting the agent")
    await agent.start()
    console.log("Ok :)")
    const secondaryDID = await agent.createNewPeerDID([], true);
    const message = new SDK.BasicMessage(
        { content: "Test Message" },
        secondaryDID,
        secondaryDID,
    );


    console.log("Sending message")
    await agent.sendMessage(message.makeMessage());
    console.log("OK")
})()
