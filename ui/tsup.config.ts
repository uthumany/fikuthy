import {defineConfig} from 'tsup';

export default defineConfig({
  entry: ['src/index.tsx'],
  format: ['esm'],
  platform: 'node',
  target: 'node22',
  dts: true,
  clean: true,
  splitting: false,
  define: {'process.env.NODE_ENV': '"production"'},
  banner: {
    js: "import {createRequire} from 'node:module';const require=createRequire(import.meta.url);",
  },
  // Release archives install only ui/dist next to the native binary. Bundle
  // runtime dependencies so that installation never relies on a repository
  // node_modules directory or a package.json in the user's home directory.
  noExternal: [/.*/],
});
