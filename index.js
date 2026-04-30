const fs = require("fs");

(async () => {
  const bytes = fs.readFileSync("output.wasm");
  const { instance } = await WebAssembly.instantiate(bytes);

  const inputs = [0n, 1n, 5n, 10n];
  for (const n of inputs) {
    const result = instance.exports.fib(n);
    console.log(`run(${n}) = ${result}`);
  }
})();

