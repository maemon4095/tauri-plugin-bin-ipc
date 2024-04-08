import { parse } from "$std/flags/mod.ts";
import { Builder, BuilderOptions } from "https://raw.githubusercontent.com/maemon4095/deno-esbuilder/release/v0.2.1/src/mod.ts";

const args = parse(Deno.args, {
    boolean: ["dev"],
});
const is_dev = args.dev;
const mode = args._[0];

const commonOptions: BuilderOptions = {
    documentFilePath: "./index.html",
    denoConfigPath: "./deno.json",
    loader: {
        ".svg": "file"
    }
};

const options: BuilderOptions = is_dev ? {
    ...commonOptions,
    minifySyntax: false,
} : {
    ...commonOptions,
    dropLabels: ["DEV"]
};

const builder = new Builder(options);

switch (mode) {
    case "serve": {
        await builder.serve();
        break;
    }
    case "build": {
        await builder.build();
        break;
    }
}