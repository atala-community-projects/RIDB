
import esbuild from 'esbuild';
import fs from 'fs';
import path from 'path';
import { NodeResolvePlugin } from '@esbuild-plugins/node-resolve';

const wasmPlugin = {
    name: 'wasm',
    setup(build) {
        build.onResolve({ filter: /\.wasm$/ }, args => {
            return { path: path.resolve(args.resolveDir, args.path), namespace: 'wasm' };
        });
        build.onLoad({ filter: /.*/, namespace: 'wasm' }, async (args) => {
            const buffer = await fs.promises.readFile(args.path);
            const base64 = buffer.toString('base64');
            return {
                contents: `export default Buffer.from("${base64}", "base64")`,
                loader: 'js',
            };
        });
    },
};

const plugins = [
    NodeResolvePlugin({
        extensions: ['.ts', '.js', '.wasm'],
        onResolved: (resolved) => {
            if (resolved.includes('node_modules')) {
                return {
                    external: true,
                }
            }
            return resolved
        },
    }),
]

const generic = {
    chunkNames: "[name]",
    assetNames: "[name]",
    entryPoints: ['src/index.ts'],
    sourcemap: false,
    bundle: true,
    platform: 'neutral',
    splitting: false,
    resolveExtensions: ['.ts', '.js', '.wasm'],
    inject: [],
    mainFields: ['module', 'main'],
    banner: {
        js:  "\nif (typeof Buffer === 'undefined') {\n  global.Buffer = require('buffer').Buffer;\n}",
    },
    define: {
        'global.Buffer': 'Buffer',
    },
    external: ['buffer']
}

// Build ES module
esbuild.build({
    ...generic,
    outfile:"build/esm/index.mjs",
    target: ['esnext'],
    format: 'esm',
    plugins: [
        wasmPlugin,
        ...plugins
    ],
}).then(() => {
    esbuild.build({
        ...generic,
        outfile:"build/cjs/index.cjs",
        target: ['es6'],
        format: 'cjs',
        plugins: [
            wasmPlugin,
            ...plugins
        ],
    }).catch((err) => {
        console.log(err)
        process.exit(1)
    });
})

    .catch((err) => {
    console.log(err)
    process.exit(1)
});

