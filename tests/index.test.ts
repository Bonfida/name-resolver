import { test, expect } from "vitest";

const URL = "http://0.0.0.0:8787/";

test("It should be running", async () => {
  const response = await fetch(URL);
  expect(response.status).toBe(200);

  const text = await response.text();
  expect(text).toBe("Visit https://bonfida.org");
});

test("URL Record", async () => {
  const response = await fetch(URL + "bonfida");
  expect(response.redirected).toBe(true);
  expect(response.url).toBe("https://www.sns.id/");
});

test("IPFS Record", async () => {
  const response = await fetch(URL + "djhksnf");
  expect(response.redirected).toBe(true);
  expect(response.url).toBe(
    "https://cloudflare-ipfs.com/ipfs/bafybeic4snhbvdwop4z5q6bzcmprrzeoph5ewgv7mackpqe2wvppkc4meu/"
  );
});
