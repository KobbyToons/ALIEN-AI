// Listens for test execution requests from background script
window.addEventListener("message", async (event) => {
  if (event.data && event.data.type === "ALIEN_EXEC_TEST") {
    const { code, testName } = event.data;
    const outputDiv = document.getElementById("test-output");
    outputDiv.textContent = `Executing test: ${testName || "Code Test"}...`;

    const logs = [];
    const origLog = console.log;
    console.log = (...args) => {
      logs.push(args.join(" "));
      origLog.apply(console, args);
    };

    let passed = true;
    let result = null;
    let error = null;
    const startTime = performance.now();

    try {
      const fn = new Function(`return (async () => { ${code} })()`);
      result = await fn();
    } catch (e) {
      passed = false;
      error = e.message;
    } finally {
      console.log = origLog;
    }

    const durationMs = Math.round((performance.now() - startTime) * 100) / 100;

    outputDiv.textContent = passed ? "Test Passed!" : `Failed: ${error}`;

    window.parent.postMessage(
      {
        type: "ALIEN_TEST_RESULT",
        passed,
        result,
        error,
        logs,
        durationMs,
      },
      "*"
    );
  }
});
