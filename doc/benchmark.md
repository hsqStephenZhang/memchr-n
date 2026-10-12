# 性能测试

## 测试数据与统计口径

输入为 16 KiB 的确定性伪随机字节，needle 数量为 1、2、3、4、6、8。
生成输入时先移除所有意外匹配，再按场景放入一个匹配字节。

| 场景 | 首次匹配位置 | 报告指标 |
| --- | --- | --- |
| `no_match` | 无匹配，搜索完整输入 | 延迟及按 16 KiB 计算的吞吐量 |
| `end_match` | 16383，搜索到最后一个字节 | 延迟及按 16 KiB 计算的吞吐量 |
| `early_match` | 255 | 仅延迟 |

比较 naive、memchr、memchr-n 和 aho-corasick 的首次匹配搜索。
memchr 对多个 needle 分组搜索后取最小有效位置；计时前检查四种算法的结果一致。

此前结果在位置 255 提前返回，却按完整 16 KiB 计算吞吐量，数值高估约 64 倍。
旧结果表已移除，应使用修复后的场景重新测量，分别比较各场景的数据。

## 运行

```sh
cargo bench -p memchr-n --bench find
```

快速校验所有场景和算法，不进行统计测量：

```sh
cargo bench -p memchr-n --bench find -- --test
```

Criterion 报告位于 `target/criterion`。保存结果时应记录 CPU、Rust 版本及编译参数；
仓库默认使用 `-C target-cpu=native`。
