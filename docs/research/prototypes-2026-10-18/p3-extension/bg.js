const api = typeof browser !== "undefined" ? browser : chrome;
const port = api.runtime.connectNative("com.breathly.proto");
port.onMessage.addListener(m => port.postMessage({type:"ack", got:m.type, size:JSON.stringify(m).length}));
port.onDisconnect.addListener(() => {});
port.postMessage({type:"ping"});
port.postMessage({type:"status"});
setTimeout(() => port.postMessage({type:"big", bytes:500000}), 1500);
setTimeout(() => port.postMessage({type:"big", bytes:1200000}), 3500);
setTimeout(() => port.postMessage({type:"up", pad:"y".repeat(2000000)}), 6000);
