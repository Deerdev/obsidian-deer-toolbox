/*
### add polyfill in ../pkg/obsidian_deer_toolbox_wasm.js
function freeStackObject() {
  heap[stack_pointer++] = undefined;
}

async function process_web_image_from_content_none_free(plugin) {
  const ret = await wasm.process_web_image_from_content(addBorrowedObject(plugin));
  return takeObject(ret);
}

export { finalizeInit, initMemory, load, getImports, freeStackObject, process_web_image_from_content_none_free }
*/

import { getImports, initMemory, load, finalizeInit, process_web_image_from_content_none_free as process_web_image_from_content, freeStackObject  } from '../pkg/obsidian_deer_toolbox_wasm.js';

export { process_web_image_from_content, freeStackObject };

export const initWithBuf = async (buf: any) => {
  const imports = getImports();

  initMemory(imports);

  const { instance, module } = await load(buf, imports);

  return finalizeInit(instance, module);
}

