import * as path from "jsr:@std/path";
import * as esbuild from "npm:esbuild@0.21";
import generateIndexFile, {
    linking,
} from "jsr:@maemon4095-esbuild-x/plugin-generate-index-file@0.6";
import { denoPlugins } from "jsr:@luca/esbuild-deno-loader@0.10";
import tailwindcss from "npm:tailwindcss";
import tailwindConfig from "./tailwind.config.js";
import postcssPlugin from "jsr:@maemon4095-esbuild-x/plugin-postcss";

const mode = Deno.args[0];
switch (mode) {
    case "dev":
    case "build":
        break;
    case undefined: {
        throw new Error("no mode was provided");
    }
    default: {
        console.log(`unrecognized mode: ${mode}`);
        Deno.exit(1);
    }
}
const distdir = path.join(import.meta.dirname!, "./dist");
const configPath = path.join(import.meta.dirname!, "./deno.json");

switch (mode) {
    case "dev": {
        const context = await createContext(mode, ["./src/index.tsx"], distdir);
        await context.watch();
        const { port } = await context.serve({ servedir: distdir, port: 1420 });

        console.log(`Serving: http://localhost:${port}`);
        break;
    }
    case "build": {
        const context = await createContext(mode, ["./src/index.tsx"], distdir);
        await context.rebuild();
        await context.dispose();
        break;
    }
}

function cleanOutdir(): esbuild.Plugin {
    return {
        name: "clean-outdir",
        setup(build) {
            const { outdir } = build.initialOptions;
            if (outdir === undefined) {
                throw new Error("outdir must be set.");
            }
            build.onStart(async () => {
                try {
                    await Deno.remove(outdir, { recursive: true });
                } catch {
                    console.log("Failed to clear outdir.");
                }
            });
        },
    };
}

async function createContext(
    mode: "build" | "dev",
    entryPoints: string[],
    outdir: string,
) {
    const outbase = "./src";
    return await esbuild.context({
        entryPoints,
        metafile: true,
        outbase,
        bundle: true,
        format: "esm",
        platform: "browser",
        jsx: "automatic",
        jsxImportSource: "preact",
        assetNames: "[dir]/[name]",
        outdir,
        sourcemap: mode !== "build",
        plugins: [
            cleanOutdir(),
            postcssPlugin({
                plugins: [
                    tailwindcss(tailwindConfig),
                ],
            }),
            generateIndexFile({
                generate: () => [
                    generate(),
                ],
            }),
            ...denoPlugins({ configPath: configPath }),
        ],
    });
}

function generate() {
    const filepath = "index.html";
    const filter = path.globToRegExp(`**/*`);
    const determineLink = (file: string) => {
        if (file.endsWith(".js")) {
            return linking.script({ defer: true, type: "module" });
        }
        if (file.endsWith(".css")) return linking.link({ rel: "stylesheet" });
    };
    return ({
        filepath,
        link: (file: string) => {
            if (filter.test(file)) {
                return determineLink(file);
            }
            return undefined;
        },
        staticFiles: [
            ...(mode === "build"
                ? []
                : [{ path: "./public/hotreload.js", link: linking.script() }]),
        ],
    });
}
