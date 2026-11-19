import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const source = resolve(root, "src-tauri/icons/icon.ico.b64");
const target = resolve(root, "src-tauri/icons/icon.ico");
const encoded = (await readFile(source, "utf8")).replace(/\\s+/g, "");
if (!/^[A-Za-z0-9+/]+={0,2}$/.test(encoded)) {
  throw new Error("A fonte do ícone do DIAMANDIS ONE está inválida.");
}
const icon = Buffer.from(encoded, "base64");
if (icon.length < 1 || icon.readUInt16LE(0) !== 0 || icon.readUInt16LE(2) !== 1) {
  throw new Error("O arquivo de ícone não possui um cabeçalho ICO válido.");
}
await mkdir(dirname(target), { recursive: true });
await writeFile(target, icon);
console.log("Ícone do DIAMANDIS ONE preparado.");
