import { invoke } from '@tauri-apps/api/core';

async function test() {
  try {
    const status = await invoke('get_status');
    console.log('Status:', status);
    
    await invoke('toggle');
    const newStatus = await invoke('get_status');
    console.log('New Status:', newStatus);
  } catch (e) {
    console.error('Error:', e);
  }
}

test();
