<script setup lang="ts">
import { computed, inject, ref } from 'vue'
import type { FileItem, StartupEntry, UninstallEntry, DuplicateGroup } from '../types'

const wizard = inject<any>('wizard')!
defineEmits<{ (e: 'done'): void; (e: 'back'): void }>()

const open = ref<Record<string, boolean>>({
  temp: true, browser: true, startup: true, large: false, dup: false, uninstall: false,
})

const report = computed(() => wizard.report)

const byCat = computed(() => {
  const g: Record<string, FileItem[]> = { system_temp: [], browser_cache: [] }
  for (const f of report.value?.files ?? []) {
    if (!g[f.category]) g[f.category] = []
    g[f.category].push(f)
  }
  return g
})

const selected = computed({
  get: () => new Set<string>(wizard.plan.files),
  set: (s: Set<string>) => { wizard.plan.files = [...s] },
})

function toggle(id: string) {
  const s = new Set<string>(wizard.plan.files)
  s.has(id) ? s.delete(id) : s.add(id)
  wizard.plan.files = [...s]
}

function toggleGroup(items: FileItem[], on: boolean) {
  const s = new Set<string>(wizard.plan.files)
  for (const it of items) on ? s.add(it.id) : s.delete(it.id)
  wizard.plan.files = [...s]
}

function isSel(id: string) { return wizard.plan.files.includes(id) }

function toggleStartup(e: StartupEntry) {
  const idx = wizard.plan.startup_toggle.findIndex((t: any) => t.id === e.id)
  if (idx >= 0) wizard.plan.startup_toggle.splice(idx, 1)
  else wizard.plan.startup_toggle.push({ id: e.id, enabled: !e.enabled })
}
function startupPending(id: string) {
  return wizard.plan.startup_toggle.some((t: any) => t.id === id)
}

function toggleUninstall(u: UninstallEntry) {
  const arr: string[] = wizard.plan.uninstall_ids
  const i = arr.indexOf(u.id)
  i >= 0 ? arr.splice(i, 1) : arr.push(u.id)
}

function fmt(bytes: number): string {
  const u = ['B', 'KB', 'MB', 'GB', 'TB']
  let i = 0; let v = bytes
  while (v >= 1024 && i < u.length - 1) { v /= 1024; i++ }
  return `${v.toFixed(v < 10 && i > 0 ? 1 : 0)} ${u[i]}`
}

function pill(risk: string) {
  return risk === 'sensitive' ? 'danger' : risk === 'review' ? 'warn' : 'ok'
}
</script>

<template>
  <section class="col" v-if="report">
    <div class="panel col">
      <h2>Revisar resultados</h2>
      <p class="dim">Marca lo que quieres limpiar. Los elementos sensibles vienen desmarcados por defecto.</p>
    </div>

    <details class="panel" :open="open.temp">
      <summary>
        Temporales y cache del sistema
        <span class="pill">{{ byCat.system_temp?.length ?? 0 }}</span>
        <button @click.prevent="toggleGroup(byCat.system_temp ?? [], true)">Marcar todo</button>
        <button @click.prevent="toggleGroup(byCat.system_temp ?? [], false)">Desmarcar</button>
      </summary>
      <table>
        <thead><tr><th></th><th>Ruta</th><th>Tamano</th><th>Riesgo</th><th>UAC</th></tr></thead>
        <tbody>
          <tr v-for="f in byCat.system_temp" :key="f.id">
            <td><input type="checkbox" :checked="isSel(f.id)" @change="toggle(f.id)" /></td>
            <td class="mono">{{ f.path }}</td>
            <td>{{ fmt(f.size) }}</td>
            <td><span class="pill" :class="pill(f.risk)">{{ f.risk }}</span></td>
            <td>{{ f.requires_elevation ? 'si' : '' }}</td>
          </tr>
        </tbody>
      </table>
    </details>

    <details class="panel" :open="open.browser">
      <summary>
        Cache de navegadores
        <span class="pill">{{ byCat.browser_cache?.length ?? 0 }}</span>
        <button @click.prevent="toggleGroup(byCat.browser_cache ?? [], true)">Marcar todo</button>
        <button @click.prevent="toggleGroup(byCat.browser_cache ?? [], false)">Desmarcar</button>
      </summary>
      <table>
        <thead><tr><th></th><th>Ruta</th><th>Tamano</th></tr></thead>
        <tbody>
          <tr v-for="f in byCat.browser_cache" :key="f.id">
            <td><input type="checkbox" :checked="isSel(f.id)" @change="toggle(f.id)" /></td>
            <td class="mono">{{ f.path }}</td>
            <td>{{ fmt(f.size) }}</td>
          </tr>
        </tbody>
      </table>
    </details>

    <details class="panel" :open="open.startup">
      <summary>Gestion de arranque <span class="pill">{{ report.startup.length }}</span></summary>
      <table>
        <thead><tr><th>Nombre</th><th>Origen</th><th>Comando</th><th>Estado</th><th>Accion</th></tr></thead>
        <tbody>
          <tr v-for="e in report.startup" :key="e.id">
            <td>{{ e.name }}</td>
            <td class="dim">{{ e.source }}</td>
            <td class="mono">{{ e.command }}</td>
            <td>
              <span class="pill" :class="e.enabled ? 'ok' : 'warn'">
                {{ e.enabled ? 'activo' : 'deshabilitado' }}
              </span>
              <span v-if="startupPending(e.id)" class="pill warn">cambio pendiente</span>
            </td>
            <td>
              <button @click="toggleStartup(e)">
                {{ e.enabled ? 'Deshabilitar' : 'Habilitar' }}
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </details>

    <details class="panel" :open="open.uninstall">
      <summary>Desinstalar programas <span class="pill">{{ report.uninstalls.length }}</span></summary>
      <table>
        <thead><tr><th></th><th>Programa</th><th>Editor</th><th>Instalado</th><th></th></tr></thead>
        <tbody>
          <tr v-for="u in report.uninstalls" :key="u.id">
            <td><input type="checkbox" :checked="wizard.plan.uninstall_ids.includes(u.id)" @change="toggleUninstall(u)" /></td>
            <td>
              {{ u.name }}
              <span v-if="u.heavy_startup" class="pill warn">arranque pesado</span>
            </td>
            <td class="dim">{{ u.publisher ?? '' }}</td>
            <td>{{ u.install_date ?? '' }}</td>
            <td><span v-if="u.estimated_size">{{ fmt(u.estimated_size) }}</span></td>
          </tr>
        </tbody>
      </table>
    </details>

    <details class="panel" :open="open.large">
      <summary>Archivos grandes <span class="pill">{{ report.large_files.length }}</span></summary>
      <table>
        <thead><tr><th></th><th>Ruta</th><th>Tamano</th></tr></thead>
        <tbody>
          <tr v-for="f in report.large_files" :key="f.id">
            <td><input type="checkbox" :checked="isSel(f.id)" @change="toggle(f.id)" /></td>
            <td class="mono">{{ f.path }}</td>
            <td>{{ fmt(f.size) }}</td>
          </tr>
        </tbody>
      </table>
    </details>

    <details class="panel" :open="open.dup">
      <summary>Duplicados <span class="pill">{{ report.duplicates.length }} grupos</span></summary>
      <div v-for="g in report.duplicates" :key="g.hash" class="col" style="margin-bottom: 8px">
        <div class="dim mono">{{ g.hash.slice(0, 12) }} · {{ fmt(g.size) }} c/u</div>
        <div v-for="(p, i) in g.paths" :key="p" class="row">
          <input type="checkbox" :checked="isSel(`dup:${g.hash}:${i}`)" @change="toggle(`dup:${g.hash}:${i}`)" :disabled="i === 0" />
          <span class="mono">{{ p }}</span>
          <span v-if="i === 0" class="pill">original</span>
        </div>
      </div>
    </details>

    <div class="row">
      <button @click="$emit('back')">Volver</button>
      <div class="grow"></div>
      <button class="primary" @click="$emit('done')">Continuar</button>
    </div>
  </section>
</template>

<style scoped>
summary { cursor: pointer; display: flex; gap: 8px; align-items: center; }
summary::-webkit-details-marker { display: none; }
summary button { margin-left: auto; }
summary button + button { margin-left: 0; }
</style>
