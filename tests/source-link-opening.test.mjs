import test from 'node:test'
import assert from 'node:assert/strict'
import fs from 'node:fs'

const repositoryPageSource = fs.readFileSync(
  'E:/vibecoding-project/skills-repository/src/pages/RepositoryPage.tsx',
  'utf8',
)
const marketPageSource = fs.readFileSync(
  'E:/vibecoding-project/skills-repository/src/pages/MarketPage.tsx',
  'utf8',
)
const tauriClientSource = fs.readFileSync(
  'E:/vibecoding-project/skills-repository/src/lib/tauri-client.ts',
  'utf8',
)
const appCommandSource = fs.readFileSync(
  'E:/vibecoding-project/skills-repository/src-tauri/src/commands/app.rs',
  'utf8',
)
const rustSourceOpenerSource = fs.readFileSync(
  'E:/vibecoding-project/skills-repository/src-tauri/src/services/source_reference.rs',
  'utf8',
)
const tauriLibSource = fs.readFileSync(
  'E:/vibecoding-project/skills-repository/src-tauri/src/lib.rs',
  'utf8',
)
const packageJsonSource = fs.readFileSync(
  'E:/vibecoding-project/skills-repository/package.json',
  'utf8',
)
const cargoTomlSource = fs.readFileSync(
  'E:/vibecoding-project/skills-repository/src-tauri/Cargo.toml',
  'utf8',
)

test('repository detail should open source via shared opener helper', () => {
  assert.ok(
    repositoryPageSource.includes("import { openSourceReference } from '../lib/tauri-client'"),
    'RepositoryPage should import source opener from tauri-client',
  )
  assert.ok(
    repositoryPageSource.includes('void openSourceReference(selectedDetail.sourceUrl!)'),
    'RepositoryPage should open source through tauri-client',
  )
  assert.ok(
    !repositoryPageSource.includes('href={selectedDetail.sourceUrl}'),
    'RepositoryPage should not use raw anchor href for source links',
  )
})

test('market page should open source via shared opener helper', () => {
  assert.ok(
    marketPageSource.includes("import { openSourceReference } from '../lib/tauri-client'"),
    'MarketPage should import source opener from tauri-client',
  )
  assert.ok(
    marketPageSource.includes('void openSourceReference(item.sourceUrl)'),
    'MarketPage should open source through tauri-client',
  )
  assert.ok(
    !marketPageSource.includes('href={item.sourceUrl}'),
    'MarketPage should not use raw anchor href for source links',
  )
})

test('tauri client should invoke a backend source opener command', () => {
  assert.ok(
    tauriClientSource.includes("invoke<void>('open_source_reference'"),
    'tauri-client should expose open_source_reference command',
  )
  assert.ok(
    appCommandSource.includes('pub fn open_source_reference'),
    'Rust command layer should expose open_source_reference',
  )
  assert.ok(
    rustSourceOpenerSource.includes('tauri_plugin_opener'),
    'Source opening service should use tauri-plugin-opener on Rust side',
  )
})

test('tauri app should register opener plugin without frontend path permissions', () => {
  assert.ok(
    tauriLibSource.includes('.plugin(tauri_plugin_opener::init())'),
    'Tauri runtime should initialize tauri-plugin-opener',
  )
  assert.ok(
    !packageJsonSource.includes('"@tauri-apps/plugin-opener"'),
    'Frontend should not depend on plugin-opener directly',
  )
  assert.ok(
    cargoTomlSource.includes('tauri-plugin-opener'),
    'Rust dependencies should include tauri-plugin-opener',
  )
})
