const esbuild = require('esbuild');
const path = require('path');

const watch = process.argv.includes('--watch');

const buildOptions = {
    entryPoints: ['src/plugin.ts'],
    bundle: true,
    outfile: 'com.deckremote.keyblock.sdPlugin/bin/plugin.js',
    platform: 'node',
    target: 'node20',
    format: 'cjs',
    external: ['@elgato/streamdeck'],
};

async function build() {
    try {
        if (watch) {
            const ctx = await esbuild.context(buildOptions);
            await ctx.watch();
            console.log('Watching for changes...');
        } else {
            await esbuild.build(buildOptions);
            console.log('Build complete!');
        }
    } catch (error) {
        console.error('Build failed:', error);
        process.exit(1);
    }
}

build();
