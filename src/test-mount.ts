import { mount } from "svelte";
import { TOOLS } from "./lib/tools";
const root = document.getElementById("results")!;
const uncaught: string[] = [];
window.addEventListener("error", (e) => uncaught.push("error: " + e.message));
window.addEventListener("unhandledrejection", () => {});
function line(t: string, fail: boolean){ const d=document.createElement("div"); d.textContent=t; if(fail)d.setAttribute("data-fail","1"); root.appendChild(d); }
for (const t of TOOLS){ const box=document.createElement("div"); box.style.display="none"; document.body.appendChild(box);
  try { mount(t.component,{target:box}); line("PASS "+t.id,false);} catch(e){ line("FAIL "+t.id+" :: "+(e as Error).message,true);} }
setTimeout(()=>{ for(const u of uncaught) line("UNCAUGHT "+u,false); root.setAttribute("data-done","1"); }, 1500);
