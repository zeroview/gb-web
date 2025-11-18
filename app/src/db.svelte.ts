import Dexie, { type EntityTable } from "dexie"

interface StateData {
  id: number,
  romHash: number,
  slot: number,
  state: ArrayBuffer
}

type DexieDB = Dexie & {
  states: EntityTable<StateData, "id">
}
export class Database {
  private db: DexieDB;

  constructor() {
    const db = new Dexie("DMG-2025") as DexieDB;
    db.version(1).stores({
      states: "++id, romHash, slot"
    })
    this.db = db;
  }

  private getCollection = (romHash: number, slot: number) => {
    return this.db.states.filter(state => state.romHash === romHash && state.slot === slot);
  }

  saveState = async (romHash: number, slot: number, state: Uint8Array) => {
    // Delete previous state
    let collection = this.getCollection(romHash, slot);
    await collection.delete();
    // Save new state
    await this.db.states.add({
      romHash,
      slot,
      state: state.buffer,
    });
  }

  getState = async (romHash: number, slot: number) => {
    let collection = this.getCollection(romHash, slot);
    let result = await collection.first();
    return result !== undefined ? new Uint8Array(result.state) : null;
  }
}

export default Database;
