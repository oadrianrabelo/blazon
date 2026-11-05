// Signs a release installer for Blazon's updater (src-tauri/src/updater.rs):
// writes <installer>.sig, the base64 Ed25519 signature of
// "blazon-update:<version>:<sha256 of the installer>".
// The private key (PKCS#8 PEM) comes from the BLAZON_UPDATE_KEY environment
// variable, a repository secret; it never touches the disk here.
//
//   node scripts/sign-update.mjs <installer.exe> <version>
import { createHash, createPrivateKey, sign } from 'node:crypto';
import { readFileSync, writeFileSync } from 'node:fs';

const [file, version] = process.argv.slice(2);
const pem = process.env.BLAZON_UPDATE_KEY;
if (!file || !version) {
  console.error('uso: node scripts/sign-update.mjs <instalador> <versão>');
  process.exit(2);
}
if (!pem) {
  console.error('BLAZON_UPDATE_KEY não está configurada: as versões novas precisam ser assinadas para a atualização automática aceitá-las.');
  process.exit(1);
}
const installer = readFileSync(file);
const hash = createHash('sha256').update(installer).digest('hex');
const message = `blazon-update:${version.replace(/^v/, '')}:${hash}`;
const signature = sign(null, Buffer.from(message), createPrivateKey(pem)).toString('base64');
writeFileSync(`${file}.sig`, signature);
console.log(`assinado: ${file}.sig (${message})`);
