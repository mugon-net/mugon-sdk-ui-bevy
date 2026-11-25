// Fails when the Bevy theme (libraries/mugon-sdk-ui-bevy/src/theme.rs) drifts from the web frontend's Mantine theme.
//
// What is checked:
// - services/frontend/theme.ts still sets primaryColor "gray" (the Bevy theme is built for that),
// - the dark palette `DARK_*` equals Mantine's `dark` colors exactly,
// - `GRAY_*` are Mantine's gray shades with the blue tint removed (neutral, and as light as the original),
// - `RADIUS` is Mantine's `md` radius.
//
// Run from the repository root after `npm ci`: node libraries/mugon-sdk-ui-bevy/scripts/check-theme-parity.mjs
import { readFileSync } from "node:fs";
import { DEFAULT_THEME } from "@mantine/core";

const root = new URL("../../../", import.meta.url);
const rust = readFileSync(new URL("libraries/mugon-sdk-ui-bevy/src/theme.rs", root), "utf8");
const web = readFileSync(new URL("services/frontend/theme.ts", root), "utf8");

const failures = [];
const check = (ok, message) => ok || failures.push(message);

check(/primaryColor:\s*"gray"/.test(web), 'services/frontend/theme.ts no longer sets primaryColor: "gray"; review the Bevy theme');

const palette = Object.fromEntries([...rust.matchAll(/pub const (\w+): Color = hex\(0x([0-9a-fA-F]{6})\);/g)].map(([, name, hex]) => [name, hex.toLowerCase()]));
const channels = (hex) => [0, 2, 4].map((offset) => parseInt(hex.slice(offset, offset + 2), 16));

for (const [name, hex] of Object.entries(palette)) {
  const [family, shade] = name.split("_");
  if (family === "DARK") {
    const expected = DEFAULT_THEME.colors.dark[Number(shade)].replace("#", "").toLowerCase();
    check(hex === expected, `${name} is #${hex}, Mantine dark.${shade} is #${expected}`);
  } else if (family === "GRAY") {
    const [red, green, blue] = channels(hex);
    const mantine = channels(DEFAULT_THEME.colors.gray[Number(shade)].replace("#", ""));
    const mean = mantine.reduce((sum, value) => sum + value, 0) / 3;
    check(red === green && green === blue, `${name} (#${hex}) is not neutral`);
    check(Math.abs(red - mean) <= 6, `${name} (#${hex}) is not close to Mantine gray.${shade} without its tint (mean ${Math.round(mean)})`);
  }
}
check(Object.keys(palette).some((name) => name.startsWith("DARK_")), "found no DARK_* colors in theme.rs: did the palette move or change format?");

const radius = rust.match(/pub const RADIUS: f32 = ([\d.]+);/)?.[1];
const expectedRadius = parseFloat(DEFAULT_THEME.radius.md.match(/([\d.]+)rem/)?.[1]) * 16;
check(Number(radius) === expectedRadius, `RADIUS is ${radius}, Mantine md radius is ${expectedRadius}px`);

if (failures.length > 0) {
  console.error(`Bevy theme drifted from the web theme:\n- ${failures.join("\n- ")}`);
  process.exit(1);
}
console.log(`Bevy theme matches the web theme (${Object.keys(palette).length} palette colors checked).`);
