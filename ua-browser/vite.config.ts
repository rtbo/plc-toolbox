import vue from '@vitejs/plugin-vue'
import { spawn } from 'node:child_process'
import { fileURLToPath } from 'node:url'
import { defineConfig, type Plugin } from 'vite'
import tailwindcss from '@tailwindcss/vite'

const repoRoot = fileURLToPath(new URL('..', import.meta.url))

function generateUaTypes(): Plugin {
  return {
    name: 'generate-ua-types',
    buildStart() {
      return new Promise<void>((resolve, reject) => {
        const child = spawn(
          'cargo',
          [
            'run',
            '--bin',
            'ua-nsc',
            '--',
            'typescript',
            '-t',
            'schema/Opc.Ua.Types.bsd',
            '-o',
            'ua-browser/src/types/ua.ts',
          ],
          { cwd: repoRoot, stdio: 'inherit' },
        )

        child.once('error', reject)
        child.once('close', (code) => {
          if (code === 0) {
            resolve()
          } else {
            reject(new Error(`ua-nsc exited with code ${code}`))
          }
        })
      })
    },
  }
}

// https://vite.dev/config/
export default defineConfig({
  plugins: [generateUaTypes(), vue(), tailwindcss()],
  resolve: {
    alias: {
      '@': '/src',
    },
  },
  server: {
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
})

