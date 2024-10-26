use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// Errors that can occur during buffer operations
#[derive(Debug)]
pub enum BufferError {
    Overflow(usize),    // Attempted to push more samples than capacity
    Underflow(usize),   // Attempted to read more samples than available
    Locked,             // Could not acquire lock on buffer
}

/// Configuration for buffer behavior
#[derive(Debug, Clone)]
pub struct BufferConfig {
    capacity: usize,            // Maximum number of samples
    overflow_strategy: OverflowStrategy,
    underflow_strategy: UnderflowStrategy,
}

/// Strategy for handling buffer overflow
#[derive(Debug, Clone)]
pub enum OverflowStrategy {
    Drop,           // Drop new samples if buffer is full
    DropOldest,     // Remove oldest samples to make room
    Error,          // Return an error if buffer would overflow
}

/// Strategy for handling buffer underflow
#[derive(Debug, Clone)]
pub enum UnderflowStrategy {
    Zero,           // Return zeros if not enough samples
    Wait,           // Wait for more samples (blocking)
    Error,          // Return an error if not enough samples
}

impl Default for BufferConfig {
    fn default() -> Self {
        Self {
            capacity: 48000,  // 1 second at 48kHz
            overflow_strategy: OverflowStrategy::DropOldest,
            underflow_strategy: UnderflowStrategy::Zero,
        }
    }
}

/// Core trait defining buffer operations
pub trait BufferManager: Send + Sync {
    /// Push new samples into the buffer
    fn push_samples(&mut self, samples: &[f32]) -> Result<(), BufferError>;
    
    /// Get samples from the buffer
    fn get_samples(&mut self, count: usize) -> Result<Vec<f32>, BufferError>;
    
    /// Clear all samples from the buffer
    fn clear(&mut self);
    
    /// Get number of samples available in buffer
    fn available(&self) -> usize;
    
    /// Get total capacity of buffer
    fn capacity(&self) -> usize;
    
    /// Get current buffer usage as a percentage
    fn usage(&self) -> f32 {
        self.available() as f32 / self.capacity() as f32
    }
}

/// Ring buffer implementation optimized for audio processing
pub struct RingBuffer {
    buffer: VecDeque<f32>,
    config: BufferConfig,
}

impl RingBuffer {
    pub fn new(config: BufferConfig) -> Self {
        Self {
            buffer: VecDeque::with_capacity(config.capacity),
            config,
        }
    }
}

impl BufferManager for RingBuffer {
    fn push_samples(&mut self, samples: &[f32]) -> Result<(), BufferError> {
        // Check if pushing these samples would exceed capacity
        if samples.len() + self.buffer.len() > self.config.capacity {
            match self.config.overflow_strategy {
                OverflowStrategy::Drop => return Ok(()),
                OverflowStrategy::DropOldest => {
                    // Remove oldest samples to make room
                    let overflow = (self.buffer.len() + samples.len()) - self.config.capacity;
                    for _ in 0..overflow {
                        self.buffer.pop_front();
                    }
                }
                OverflowStrategy::Error => {
                    return Err(BufferError::Overflow(samples.len()));
                }
            }
        }
        
        // Push new samples
        self.buffer.extend(samples.iter().copied());
        Ok(())
    }

    fn get_samples(&mut self, count: usize) -> Result<Vec<f32>, BufferError> {
        if count > self.buffer.len() {
            match self.config.underflow_strategy {
                UnderflowStrategy::Zero => {
                    let mut result = Vec::with_capacity(count);
                    // Add available samples
                    result.extend(self.buffer.drain(..));
                    // Fill remainder with zeros
                    result.resize(count, 0.0);
                    Ok(result)
                }
                UnderflowStrategy::Error => {
                    Err(BufferError::Underflow(count - self.buffer.len()))
                }
                UnderflowStrategy::Wait => {
                    // In a real implementation, this would use condition variables
                    // For now, we'll just error
                    Err(BufferError::Underflow(count - self.buffer.len()))
                }
            }
        } else {
            Ok(self.buffer.drain(..count).collect())
        }
    }

    fn clear(&mut self) {
        self.buffer.clear();
    }

    fn available(&self) -> usize {
        self.buffer.len()
    }

    fn capacity(&self) -> usize {
        self.config.capacity
    }
}

/// Thread-safe buffer implementation for multi-threaded audio processing
pub struct ThreadSafeBuffer {
    inner: Arc<Mutex<RingBuffer>>,
}

impl ThreadSafeBuffer {
    pub fn new(config: BufferConfig) -> Self {
        Self {
            inner: Arc::new(Mutex::new(RingBuffer::new(config))),
        }
    }
}

impl BufferManager for ThreadSafeBuffer {
    fn push_samples(&mut self, samples: &[f32]) -> Result<(), BufferError> {self.inner.lock().map_err(|_| BufferError::Locked)?.push_samples(samples)}
    fn get_samples(&mut self, count: usize) -> Result<Vec<f32>, BufferError> {self.inner.lock().map_err(|_| BufferError::Locked)?.get_samples(count)}
    fn clear(&mut self) {if let Ok(mut buffer) = self.inner.lock() {buffer.clear();}}
    fn available(&self) -> usize {self.inner.lock().map(|buffer| buffer.available()).unwrap_or(0)}
    fn capacity(&self) -> usize {self.inner.lock().map(|buffer| buffer.capacity()).unwrap_or(0)}

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer_basic() {
        let config = BufferConfig {
            capacity: 1000,
            overflow_strategy: OverflowStrategy::Error,
            underflow_strategy: UnderflowStrategy::Error,
        };
        let mut buffer = RingBuffer::new(config);

        // Test pushing and getting samples
        let samples = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        buffer.push_samples(&samples).unwrap();
        assert_eq!(buffer.available(), 5);

        let retrieved = buffer.get_samples(3).unwrap();
        assert_eq!(retrieved, vec![1.0, 2.0, 3.0]);
        assert_eq!(buffer.available(), 2);
    }

    #[test]
    fn test_overflow_strategies() {
        // Test DropOldest strategy
        let config = BufferConfig {
            capacity: 3,
            overflow_strategy: OverflowStrategy::DropOldest,
            underflow_strategy: UnderflowStrategy::Error,
        };
        let mut buffer = RingBuffer::new(config);

        buffer.push_samples(&[1.0, 2.0, 3.0]).unwrap();
        buffer.push_samples(&[4.0, 5.0]).unwrap();
        
        let samples = buffer.get_samples(3).unwrap();
        assert_eq!(samples, vec![3.0, 4.0, 5.0]);
    }

    #[test]
    fn test_underflow_strategies() {
        // Test Zero strategy
        let config = BufferConfig {
            capacity: 5,
            overflow_strategy: OverflowStrategy::Error,
            underflow_strategy: UnderflowStrategy::Zero,
        };
        let mut buffer = RingBuffer::new(config);

        buffer.push_samples(&[1.0, 2.0]).unwrap();
        let samples = buffer.get_samples(4).unwrap();
        assert_eq!(samples, vec![1.0, 2.0, 0.0, 0.0]);
    }
}