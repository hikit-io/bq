#BQ

1. 避免 `Box<Trait>` 动态分发:
   创建 `trait` 对象的规范方法是`Box<Trait>`，但大多数代码都可以使用`&mut Trait`，它也具有动态分派但节省了分配。如果您绝对需要所有权，请使用Box，但大多数用例都可以使用`&Trait`或`&mut Trait`

2. 使用枚举替代泛型，泛型会带来动态分发。
```rust
enum Car {
    Bwm(Bwm),
    Benz(Benz),
}

struct Bwm {}

struct Benz {}

```
3. 在大量数据处理时，拷贝对性能对影响相比之下较小，可以使用并行框架处理。
4. 复杂大数据处理才使用`ndarray`，否则仅使用`rayon`即可。
5. 每次使用`collect`必须至少会迭代整个集合一次，所以最好只 `collect` 一次。
6. 尽可能地使用引用，避免过多的 Clone 。因为Clone 可能伴随内存分配。
7. 使用 Unsafe 方法消除一些不必要的安全检查。