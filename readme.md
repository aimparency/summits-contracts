# summits contract

A smart contract for summits for the near blockchain. 

## dev setup

Create a local near blockchain with kurtosis.
Build, deploy and init the contract.

### create a local near blockchain with kurtosis
For development, a localnet setup is recommended. 

Use Kurtosis to run a complete near environment (including e.g. wallet web service, explorer, indexer-for-explorer, contract helper). 
- Install kurtosis on your system. 
- Clone `git@github.com:kurtosis-tech/near-kurtosis-module.git` cd into it and run `./launch-local-near-cluster.sh`. 
- Follow the "ACTION" instructions from the ouput. You may want to redirect the output into a file, because we will need it. 

### build
run `./build.sh`

### deploy
source `local-dev-deploy.src`

