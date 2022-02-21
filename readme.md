#summits contract
A smart contract for summits for the near blockchain. 
Allows creation and editing of projects (nodes) and flows (edges). 

## build
Run `./build.sh`

## deploy

### optional: setup a local NEAR environment with kurtosis
You can use kurtosis to run a complete near environment (including e.g. wallet web service, explorer, indexer-for-explorer, contract helper). 
- Install kurtosis on your system. ([kurtosis installation guide](https://docs.kurtosistech.com/installation.html))
- Clone `git@github.com:kurtosis-tech/near-kurtosis-module.git` cd into it and run `./launch-local-near-cluster.sh`. 
- Follow the "ACTION" instructions from the ouput: set the environment variables and create the local_near alias. 

### deploy contract to a random dev-account
First set local_near either by following the kurtosis guide above or by setting
`alias local_near=...` where `...` could be `near`. 

Then source the deployment script: `source deploy-newly-and-init.source` in order to dev-deploy and init the contract to a new random dev-account. 

For subsequent deployments to the same account you can source `deploy.source`.

