const CONTRACTS = [
  {
    used: 24000,
    required: 24000,
  },
  {
    used: 30000,
    required: 100000,
  },
  {
    used: 23000,
    required: 28000,
  },
  {
    used: 1400000,
    required: 1750000,
  },
];

const runContract = (contractNumber: number, gasLimit: number) => {
  const { used, required } = CONTRACTS[contractNumber];
  if (gasLimit >= required) {
    return { result: "ok", used };
  }
  return { result: "oog", used: gasLimit };
};

const MAX_GAS = 15000000;
const estimate = (contractNumber: number) => {
  let higherBound = MAX_GAS;
  let midBound = higherBound;
  let lowerBound = 21000;
  let iterations = 0;
  while (true) {
    const { result, used } = runContract(contractNumber, midBound);
    iterations++;
    console.log(
      `runContract ${contractNumber}[${iterations}x, low: ${lowerBound}, mid: ${midBound} high: ${higherBound}] => result ${result}: used ${used}`
    );
    if (result == "ok") {
      higherBound = midBound;
      if (midBound == MAX_GAS) {
        midBound = Math.min(used * 3, lowerBound + (higherBound - lowerBound) / 2);
        lowerBound = Math.max(used, lowerBound);
      } else {
        midBound = lowerBound + (higherBound - lowerBound) / 2;
      }
    } else {
      if (lowerBound == higherBound) {
        process.exit(1);
      }
      if (higherBound >= MAX_GAS) {
        return {
          iterations,
          gas: higherBound,
          result,
        };
      }
      // oog
      lowerBound = midBound;
      midBound = midBound + (higherBound - midBound) / 2;
    }
    if (higherBound - lowerBound <= higherBound / 10) {
      return {
        iterations,
        gas: higherBound,
        result,
      };
    }
  }
};

const main = () => {
  for (let contractNumber = 0; contractNumber < CONTRACTS.length; contractNumber++) {
    console.log(
      `========== Contract ${contractNumber}: gas ${CONTRACTS[contractNumber].used}, required: ${CONTRACTS[contractNumber].required}`
    );
    const { iterations, gas, result } = estimate(contractNumber);
    console.log(`${contractNumber}: gas ${gas}, iterations: ${iterations}: ${result}`);
  }
};

main();
