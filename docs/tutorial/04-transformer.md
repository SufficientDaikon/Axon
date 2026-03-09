# Tutorial 4: Building a Transformer

Build a transformer encoder from scratch in Axon to understand self-attention,
multi-head attention, and the full transformer architecture.

**Prerequisites**: [Tutorial 3: MNIST Classifier](03-mnist-classifier.md)

---

## Architecture Overview

A transformer encoder block consists of:

1. **Multi-Head Self-Attention**
2. **Layer Normalization** + residual connection
3. **Feed-Forward Network** (two linear layers with activation)
4. **Layer Normalization** + residual connection

---

## Step 1: Scaled Dot-Product Attention

The fundamental building block of transformers:

```axon
fn scaled_dot_product_attention(
    query: Tensor<Float32, [?, ?, ?]>,    // [batch, seq_len, d_k]
    key: Tensor<Float32, [?, ?, ?]>,      // [batch, seq_len, d_k]
    value: Tensor<Float32, [?, ?, ?]>,    // [batch, seq_len, d_v]
) -> Tensor<Float32, [?, ?, ?]> {
    let d_k = query.shape[2] as Float32;

    // Attention scores: Q @ K^T / sqrt(d_k)
    let scores = (query @ key.transpose()) / d_k.sqrt();

    // Softmax over the last dimension
    let weights = softmax(scores, dim: 2);

    // Weighted sum of values
    weights @ value
}
```

---

## Step 2: Multi-Head Attention

Split the model dimension into multiple heads for parallel attention:

```axon
use std::nn::Linear;

struct MultiHeadAttention {
    num_heads: Int32,
    d_model: Int32,
    d_k: Int32,
    w_query: Linear<512, 512>,
    w_key: Linear<512, 512>,
    w_value: Linear<512, 512>,
    w_output: Linear<512, 512>,
}

impl MultiHeadAttention {
    fn new(d_model: Int32, num_heads: Int32) -> MultiHeadAttention {
        let d_k = d_model / num_heads;
        MultiHeadAttention {
            num_heads,
            d_model,
            d_k,
            w_query: Linear::new(),
            w_key: Linear::new(),
            w_value: Linear::new(),
            w_output: Linear::new(),
        }
    }

    fn forward(
        &self,
        query: Tensor<Float32, [?, ?, 512]>,
        key: Tensor<Float32, [?, ?, 512]>,
        value: Tensor<Float32, [?, ?, 512]>,
    ) -> Tensor<Float32, [?, ?, 512]> {
        let batch_size = query.shape[0];
        let seq_len = query.shape[1];

        // Project Q, K, V
        let q = self.w_query.forward(query);
        let k = self.w_key.forward(key);
        let v = self.w_value.forward(value);

        // Reshape to [batch, num_heads, seq_len, d_k]
        let q = q.reshape([batch_size, seq_len, self.num_heads, self.d_k])
                  .permute([0, 2, 1, 3]);
        let k = k.reshape([batch_size, seq_len, self.num_heads, self.d_k])
                  .permute([0, 2, 1, 3]);
        let v = v.reshape([batch_size, seq_len, self.num_heads, self.d_k])
                  .permute([0, 2, 1, 3]);

        // Attention per head
        let attn = scaled_dot_product_attention(q, k, v);

        // Concatenate heads
        let concat = attn.permute([0, 2, 1, 3])
                         .reshape([batch_size, seq_len, self.d_model]);

        // Final projection
        self.w_output.forward(concat)
    }
}
```

---

## Step 3: Feed-Forward Network

Two linear layers with GELU activation:

```axon
struct FeedForward {
    linear1: Linear<512, 2048>,
    linear2: Linear<2048, 512>,
}

impl FeedForward {
    fn new(d_model: Int32, d_ff: Int32) -> FeedForward {
        FeedForward {
            linear1: Linear::new(),
            linear2: Linear::new(),
        }
    }

    fn forward(&self, x: Tensor<Float32, [?, ?, 512]>) -> Tensor<Float32, [?, ?, 512]> {
        let h = gelu(self.linear1.forward(x));
        self.linear2.forward(h)
    }
}
```

---

## Step 4: Transformer Encoder Block

Combine attention and feed-forward with residual connections and layer norm:

```axon
use std::nn::LayerNorm;

struct TransformerBlock {
    attention: MultiHeadAttention,
    feed_forward: FeedForward,
    norm1: LayerNorm,
    norm2: LayerNorm,
}

impl TransformerBlock {
    fn new(d_model: Int32, num_heads: Int32, d_ff: Int32) -> TransformerBlock {
        TransformerBlock {
            attention: MultiHeadAttention::new(d_model, num_heads),
            feed_forward: FeedForward::new(d_model, d_ff),
            norm1: LayerNorm::new(d_model),
            norm2: LayerNorm::new(d_model),
        }
    }

    fn forward(&self, x: Tensor<Float32, [?, ?, 512]>) -> Tensor<Float32, [?, ?, 512]> {
        // Self-attention + residual + norm
        let attn_out = self.attention.forward(x, x, x);
        let h = self.norm1.forward(x + attn_out);

        // Feed-forward + residual + norm
        let ff_out = self.feed_forward.forward(h);
        self.norm2.forward(h + ff_out)
    }
}
```

---

## Step 5: Positional Encoding

Add position information since attention is permutation-invariant:

```axon
fn positional_encoding(seq_len: Int32, d_model: Int32) -> Tensor<Float32, [?, ?]> {
    let pe = zeros([seq_len, d_model]);

    for pos in 0..seq_len {
        for i in 0..(d_model / 2) {
            let angle = pos as Float32 / (10000.0).pow(2.0 * i as Float32 / d_model as Float32);
            pe[pos][2 * i] = angle.sin();
            pe[pos][2 * i + 1] = angle.cos();
        }
    }

    pe
}
```

---

## Step 6: Full Transformer Encoder

Stack multiple transformer blocks into a complete encoder:

```axon
use std::nn::{Embedding, Linear, Module};

struct TransformerEncoder {
    embedding: Embedding,
    layers: Vec<TransformerBlock>,
    classifier: Linear<512, 10>,
    d_model: Int32,
}

impl TransformerEncoder {
    fn new(
        vocab_size: Int32,
        d_model: Int32,
        num_heads: Int32,
        num_layers: Int32,
        d_ff: Int32,
        num_classes: Int32,
    ) -> TransformerEncoder {
        let mut layers = Vec::new();
        for _ in 0..num_layers {
            layers.push(TransformerBlock::new(d_model, num_heads, d_ff));
        }

        TransformerEncoder {
            embedding: Embedding::new(vocab_size, d_model),
            layers,
            classifier: Linear::new(),
            d_model,
        }
    }
}

impl Module for TransformerEncoder {
    fn forward(&self, tokens: Tensor<Int64, [?, ?]>) -> Tensor<Float32, [?, 10]> {
        let seq_len = tokens.shape[1];

        // Token embedding + positional encoding
        let x = self.embedding.forward(tokens);
        let pe = positional_encoding(seq_len, self.d_model);
        let mut h = x + pe;

        // Pass through transformer blocks
        for layer in &self.layers {
            h = layer.forward(h);
        }

        // Classification: use [CLS] token (first position)
        let cls = h[.., 0, ..];   // [batch, d_model]
        self.classifier.forward(cls)
    }
}
```

---

## Step 7: Training

```axon
use std::optim::AdamW;
use std::loss::cross_entropy;

fn main() {
    println("=== Transformer Encoder ===\n");

    // Hyperparameters
    let vocab_size = 10000;
    let d_model = 512;
    let num_heads = 8;
    let num_layers = 6;
    let d_ff = 2048;
    let num_classes = 10;

    // Create model
    let mut model = TransformerEncoder::new(
        vocab_size, d_model, num_heads, num_layers, d_ff, num_classes,
    );

    let mut optimizer = AdamW::new(
        model.parameters(),
        lr: 0.0001,
        weight_decay: 0.01,
    );

    println("Model: {} parameters", model.param_count());
    println("Config: d_model={}, heads={}, layers={}\n", d_model, num_heads, num_layers);

    // Training loop
    let epochs = 20;
    for epoch in 0..epochs {
        let mut total_loss = 0.0;
        let mut num_batches = 0;

        for batch in &train_loader {
            let (tokens, labels) = batch;

            let logits = model.forward(tokens);
            let loss = cross_entropy(logits, labels);

            loss.backward();
            optimizer.step();
            optimizer.zero_grad();

            total_loss += loss.item();
            num_batches += 1;
        }

        let avg_loss = total_loss / num_batches as Float32;
        println("Epoch {:>2}/{}: loss = {:.4}", epoch + 1, epochs, avg_loss);
    }
}
```

---

## Step 8: Using the Built-in Transformer

Axon's stdlib includes pre-built transformer components:

```axon
use std::nn::{TransformerEncoder as TE, MultiHeadAttention};

fn main() {
    // One-liner transformer encoder
    let encoder = TE::new(
        d_model: 512,
        num_heads: 8,
        num_layers: 6,
        d_ff: 2048,
        dropout: 0.1,
    );

    let input: Tensor<Float32, [?, 128, 512]> = randn([32, 128, 512]);
    let output = encoder.forward(input);
    println("Output shape: {}", output.shape);   // [32, 128, 512]
}
```

---

## Architecture Diagram

```
Input Tokens
     │
     ▼
┌─────────────┐
│  Embedding   │
│  + Pos Enc   │
└─────┬───────┘
      │
      ▼  ×N layers
┌─────────────────────┐
│  Multi-Head Attn    │
│  + Residual + Norm  │
├─────────────────────┤
│  Feed-Forward       │
│  + Residual + Norm  │
└─────────┬───────────┘
          │
          ▼
┌─────────────┐
│  Classifier  │
│  (Linear)    │
└─────────────┘
          │
          ▼
      Logits [?, num_classes]
```

---

## Key Concepts Covered

| Concept              | Implementation                        |
| -------------------- | ------------------------------------- |
| Self-attention       | `Q @ K^T / sqrt(d_k)`, softmax, `@ V` |
| Multi-head           | Reshape → parallel attention → concat |
| Residual connections | `x + sublayer(x)`                     |
| Layer normalization  | `LayerNorm`                           |
| Positional encoding  | Sinusoidal `sin`/`cos`                |
| Classification       | [CLS] token → Linear                  |

---

## See Also

- [Tensor Guide](../guide/tensors.md) — tensor operations in depth
- [GPU Programming](../guide/gpu-programming.md) — train on GPU
- [PyTorch Migration](../migration/from-pytorch.md) — compare with PyTorch transformers
