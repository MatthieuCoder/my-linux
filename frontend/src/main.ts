// @ts-ignore
globalThis.DEBUG = false;
// @ts-ignore
import { V86Starter } from "v86";
import { Terminal } from 'xterm';
import { FitAddon } from 'xterm-addon-fit';
import "v86/v86.css";
import v86Wasm from "v86/build/v86.wasm?url";
import bios from "v86/bios/seabios.bin?url";
import cdrom from "../../build/output/images/rootfs.iso9660?url";
import "./style.css";

let emulator = new V86Starter({
  wasm_path: v86Wasm,
  memory_size: 512 * 1024 * 1024,
  vga_memory_size: 8 * 1024 * 1024,
  screen_container: document.getElementById("screen_container"),
  bios: { url: bios },
  cdrom: { url: cdrom },
  autostart: true,
  fastboot: true,
  disable_keyboard: true,
  disable_mouse: true,
  disable_speaker: true,
  network_relay_url: "wss://ws-net.matthieu-dev.xyz/router"
});

var term = new Terminal();
let fitAddon = new FitAddon();
term.loadAddon(fitAddon);
term.open(document.getElementById('terminal')!);
fitAddon.fit();

window.addEventListener('resize', () => fitAddon.fit());
setInterval(() => fitAddon.fit(), 5000);

term["onData"](function(data) {
      emulator.serial0_send(data);
});
emulator.add_listener("serial0-output-char", function(chr: string)
{
    term.write(chr);
}, this);