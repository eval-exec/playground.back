## Uniswap v2 学习/实践

1. 准备工作：

- 阅读白皮书： https://uniswap.org/whitepaper.pdf
- 阅读 uniswap docs： https://docs.uniswap.org/protocol/V2/introduction
- 阅读 v2-core, v2-periphery 的代码
- 准备测试环境
    - ganache-cli：一个支持 EVM 的本地测试节点，可以快速的在这上面部署合约
    - abigen： go-ethereum 官方的 abigen 工具，用于生成合约的 bytecode

2. 部署合约

- 部署 ERC20 Token： token.DeployToken()
- 部署 factory: core.DeployUniswapV2Router02()
- 部署 router： periphery.DeployUniswapV2Router02()

3. 添加流动性

```go
        hash0 = addLiquidity(a, ctx, ss.router02, ss.contractAddressCat, ss.contractAddressWBNB, ether100)
hash1 = addLiquidity(a, ctx, ss.router02, ss.contractAddressCat, ss.contractAddressDog, ether100)
hash2 = addLiquidity(a, ctx, ss.router02, ss.contractAddressDog, ss.contractAddressWBNB, ether100)
a.NoError(waitTxs(a, ctx, []common.Hash{hash0, hash1, hash2}))
```

4. 为`flashswap`刻意地制造 LP 之间的价格差

```go
router.SwapExactTokensForTokens()
```

5. 执行两个 LP 之间的套利交易

```go
flashSwap.TestFlashSwap()
```

6. 执行三角套利交易

```go
triangle.TestTriangleFlash()
```

## 遇到的问题

1. 在部署 router 后，router 无法完成诸如添加流动性之类的合约操作。

因为，在部署 router 之前，需要修改`UniswapV2Library.sol` 里面的与计算 pair 相关联的 hash（需要用到 Pair 合约的 bytecode）, 没有这个 hash, rouuter 就无法通过两个
token 的地址获得对应的 pair 的地址。 这是计算 pair 相关联 hash 的 go 代码。

```go
    bin, err := hexutil.Decode(core.UniswapV2PairMetaData.Bin)
a.NoError(err)

hash := crypto.Keccak256Hash(bin)
log.Println()
log.Println(hash)

```

2. 执行 IUniswapV2Pair().swap()时，死锁了。 因为`swap`的函数签名中有 lock 修饰

```solidity
  function swap(uint amount0Out, uint amount1Out, address to, bytes calldata data) external lock {}
```

所以，在一个交易中，一个 pair 只能 swap 一次。

3. 在完成三角套利的交易中，只要 4 次 transfer 交易就可以完成套利了，但是我第一次的交易方式不是最优的，用了 6 次交易才完成，这样会额外消耗 gas。 四次交易应该分别是：

```
pair0 -> 套现合约 -> pair1 -> pair2
  ^                            |
  |                            |
  +----------------------------+

```