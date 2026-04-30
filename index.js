const fs = require("fs");

(async () => {
  const bytes = fs.readFileSync("output.wasm");
  const { instance } = await WebAssembly.instantiate(bytes);

  const inputs = [30n];
  for (const n of inputs) {
    const result = instance.exports.fib(n);
    console.log(`fib(${n}) = ${result}`);
  }
})();

