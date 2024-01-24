import { Hono, Context } from "hono";
import { logger } from "hono/logger";
import {
  deserializeRecord,
  deserializeRecordV2Content,
  getDomainKeySync,
  getRecordKeySync,
  getRecordV2Key,
  NameRegistryState,
  Record,
} from "@bonfida/spl-name-service";
import { Connection } from "@solana/web3.js";
import { Record as SnsRecord } from "@bonfida/sns-records";

const RECORDS = [
  Record.Url,
  Record.IPFS,
  Record.ARWV,
  Record.SHDW,
  Record.A,
  Record.CNAME,
];

const PREFIX = new Map([
  [Record.IPFS, "https://cloudflare-ipfs.com/ipfs/"],
  [Record.ARWV, "https://arweave.net/"],
  [Record.SHDW, "https://shdw-drive.genesysgo.net/"],
  [Record.CNAME, "http://"],
  [Record.A, "http://"],
]);

const ERROR_URL = "https://sol-domain.org";

const getConnection = (c: Context<any>) => {
  return new Connection(c.env?.RPC_URL as string, "processed");
};

const formatResponse = (value: string, record: Record) => {
  if (record === Record.Url) {
    return value;
  }
  return PREFIX.get(record) + value;
};

const app = new Hono();

app.use("*", logger());

app.get("/", (c) => c.text("Visit https://bonfida.org"));

app.get("/:domain", async (c) => {
  try {
    const { domain } = c.req.param();
    const connection = getConnection(c);

    const recordKeys = RECORDS.map((e) => getRecordKeySync(domain, e));
    const recordV2Keys = RECORDS.map((e) => getRecordV2Key(domain, e));

    const { registry, nftOwner } = await NameRegistryState.retrieve(
      connection,
      getDomainKeySync(domain).pubkey
    );
    const owner = nftOwner || registry.owner;

    const infos = await connection.getMultipleAccountsInfo([
      ...recordV2Keys,
      ...recordKeys,
    ]);

    const des = infos.map((e, rawIdx) => {
      if (!e?.data) return undefined;
      try {
        const idx = rawIdx % RECORDS.length;
        if (rawIdx >= RECORDS.length) {
          // Record V1
          return deserializeRecord(
            NameRegistryState.deserialize(e.data),
            RECORDS[idx],
            recordKeys[idx]
          );
        } else {
          // Record V2
          const record = SnsRecord.deserialize(e.data);
          if (record.getStalenessId().equals(owner.toBuffer())) {
            return deserializeRecordV2Content(
              record.getContent(),
              RECORDS[idx]
            );
          }
        }
      } catch (err) {
        console.error(err);
      }
    });

    const index = des.findIndex((e) => e !== undefined);

    if (index === -1) {
      return c.redirect(ERROR_URL, 301);
    }

    const result = des[index]!;
    const record = RECORDS[index % RECORDS.length];

    return c.redirect(formatResponse(result, record), 301);
  } catch (err) {
    console.error(err);
    return c.redirect(ERROR_URL, 301);
  }
});

export default app;
