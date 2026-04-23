const games: { [key: string]: string } = {
    "Yume 2kki": "2kki",
    "Ultra Violet": "ultraviolet",
    "Dream Genie": "genie",
    "Yume Tsushin": "tsushin",
    "Love You": "loveyou",
    "[COLD]": "cold",
    "nostAlgic": "nostalgic",
    "Muma|Rope": "muma",
    "Yume Nikki": "yume",
    "Oversomnia": "oversomnia",
    "Uneven Dream": "unevendream",
    "FOG": "fog",
    "Amillusion": "amillusion",
    "Someday": "someday",
    "Deep Dreams": "deepdreams",
    "Scary Nikki": "scarynikki",
    "Okuri": "okuri",
};

const encoder = new TextEncoder();

const copies: [string, string][] = [];

const batch = parseInt(prompt("Batch?")!);
const entries = prompt("Entries?")!.replace(/"([^"]*)"/g, match => match.replace(/\n/g, '\\n')).split("\n");

console.clear();
for (const entry of entries) {
    const [name, game_name, author, description, condition, requirements, notes, points, files] = entry.split("\t");

    console.clear();
    console.log("Name:", name);
    console.log("Game:", game_name);
    console.log("Description:", description);
    console.log("Condition:", condition);
    const badge_id = prompt("ID?");
    if (!badge_id) break;

    const game_id = games[game_name];

    const lines = ["[badge]"];
    if (files.includes(".gif"))
        lines.push("animated = true");
    lines.push("map = ");
    lines.push(`points = ${points}`);
    lines.push(`art = "${author}"`);
    lines.push("\n[conditions]");
    if (notes)
        lines.push(`# ${notes}`);
    lines.push(`default = "${requirements}"`);
    lines.push("\n[lang.en]");
    lines.push(`name = "${name.replaceAll("\"", "\\\"")}"`);
    lines.push(`description = "${description.replaceAll("\"", "\\\"")}"`);
    lines.push(`condition = "${condition.replaceAll("\"", "\\\"")}"`);
    lines.push("");

    const path = `badges/${batch}/${game_id}`;
    Deno.mkdirSync(path, { recursive: true });
    Deno.writeFileSync(`${path}/${badge_id}.toml`, encoder.encode(lines.join("\n")));

    for (const file of files.replace(/"/g, '').split("\n")) {
        copies.push([file, badge_id])
    }
}

const commands = [];
for (const [sources, dest] of copies) {
    for (const source of sources.split("\\n")) {
        const parts = source.split(".");
        const extension = parts[parts.length - 1];

        commands.push(`cp '${source.trim().replaceAll("'", "\\'")}' '${dest}.${extension}'`);
    }
}

console.log(commands.join("\n"));
