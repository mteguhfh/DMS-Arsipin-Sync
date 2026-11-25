import { readFile, writeFile, readdir } from 'node:fs/promises'
import path from 'node:path'

const root = process.argv[2]
if (!root) {
  console.error('Usage: node scripts/merge-updater-manifests.mjs <artifacts-dir>')
  process.exit(1)
}

const found = []
async function walk(dir) {
  for (const entry of await readdir(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name)
    if (entry.isDirectory()) await walk(full)
    else if (entry.name === 'latest.json') found.push(full)
  }
}
await walk(root)

if (found.length === 0) throw new Error(`No latest.json found under ${root}`)

const merged = {}
for (const file of found) {
  const content = JSON.parse(await readFile(file, 'utf8'))
  const platform = Object.keys(content.platforms ?? {})[0]
  if (!platform) throw new Error(`${file} has no platforms entry`)
  merged.version = content.version
  merged.notes = content.notes
  merged.pub_date = content.pub_date
  merged.platforms = { ...merged.platforms, ...content.platforms }
  console.log(`Merged ${platform} from ${file}`)
}

const output = path.join(root, 'latest.json')
await writeFile(output, `${JSON.stringify(merged, null, 2)}\n`)
console.log(`Wrote ${output}`)
