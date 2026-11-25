import { copyFile, mkdir, readFile, readdir, writeFile } from 'node:fs/promises'
import path from 'node:path'

const [, , artifactsRoot = 'artifacts', outputDir = 'release-assets'] = process.argv

const packageJson = JSON.parse(await readFile('package.json', 'utf8'))
const version = packageJson.version
const repo = process.env.GITHUB_REPOSITORY ?? 'mteguhfh/DMS-Arsipin-Sync'
const tag = process.env.GITHUB_REF_NAME ?? `v${version}`

const files = []
async function walk(dir) {
  for (const entry of await readdir(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name)
    if (entry.isDirectory()) await walk(full)
    else files.push(full)
  }
}
await walk(artifactsRoot)

function pick(label, predicate) {
  const matches = files.filter(predicate)
  if (matches.length !== 1) {
    throw new Error(`Expected one ${label}, found ${matches.length}`)
  }
  return matches[0]
}

const targets = [
  {
    platform: 'linux-x86_64',
    target: 'x86_64-unknown-linux-gnu',
    asset: `DMS.Sync_${version}_amd64.AppImage`,
    match: file => file.includes('x86_64-unknown-linux-gnu') && file.endsWith('.AppImage'),
  },
  {
    platform: 'windows-x86_64',
    target: 'x86_64-pc-windows-msvc',
    asset: `DMS.Sync_${version}_x64-setup.exe`,
    match: file =>
      file.includes('x86_64-pc-windows-msvc') && file.includes('/nsis/') && file.endsWith('-setup.exe'),
  },
  {
    platform: 'darwin-x86_64',
    target: 'x86_64-apple-darwin',
    asset: `DMS.Sync_${version}_x64.app.tar.gz`,
    match: file => file.includes('x86_64-apple-darwin') && file.endsWith('.app.tar.gz'),
  },
  {
    platform: 'darwin-aarch64',
    target: 'aarch64-apple-darwin',
    asset: `DMS.Sync_${version}_aarch64.app.tar.gz`,
    match: file => file.includes('aarch64-apple-darwin') && file.endsWith('.app.tar.gz'),
  },
]

await mkdir(outputDir, { recursive: true })

const platforms = {}
for (const target of targets) {
  const source = pick(target.platform, target.match)
  const signatureFile = pick(`${target.platform} signature`, file =>
    file.includes(target.target) && path.basename(file) === `${path.basename(source)}.sig`,
  )

  await copyFile(source, path.join(outputDir, target.asset))
  await copyFile(signatureFile, path.join(outputDir, `${target.asset}.sig`))

  const signature = (await readFile(signatureFile, 'utf8')).replace(/\r?\n$/, '')
  const url = `https://github.com/${repo}/releases/download/${encodeURIComponent(tag)}/${encodeURIComponent(target.asset)}`
  platforms[target.platform] = { signature, url }
  console.log(`Prepared ${target.platform}: ${target.asset}`)
}

const manifest = {
  version,
  notes: `Auto-update to ${version}`,
  pub_date: new Date().toISOString(),
  platforms,
}

const manifestPath = path.join(outputDir, 'latest.json')
await writeFile(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`)
console.log(`Wrote ${manifestPath}`)
