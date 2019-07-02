const path = require('path')
const tape = require('tape')

const { Diorama, tapeExecutor, backwardCompatibilityMiddleware } = require('@holochain/diorama')

process.on('unhandledRejection', error => {
  // Will print "unhandledRejection err is not defined"
  console.error('got unhandledRejection:', error);
});

const dnaPath = path.join(__dirname, "../dist/holochain-mutual-credit-clearing.dna.json")
const dna = Diorama.dna(dnaPath, 'holochain-mutual-credit-clearing')

const diorama = new Diorama({
  instances: {
    amy: dna,
    brad: dna,
  },
  bridges: [],
  debugLog: false,
  executor: tapeExecutor(require('tape')),
  middleware: backwardCompatibilityMiddleware,
})

require('./scenarios')(diorama.registerScenario)

diorama.run()

