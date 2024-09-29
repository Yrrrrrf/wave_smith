import matplotlib.pyplot as plt
import numpy as np

# Read the sample values from the file
with open('morse_samples.txt', 'r') as file:
    samples = [float(line.strip()) for line in file]

# Create an array of time points
time = np.arange(len(samples)) / 44100  # Assuming 44.1kHz sample rate, adjust if different

# Create the plot
plt.figure(figsize=(15, 5))
plt.plot(time, samples)
plt.title('Morse Code Audio Waveform')
plt.xlabel('Time (seconds)')
plt.ylabel('Amplitude')
plt.ylim(-1.1, 1.1)  # Set y-axis limits
plt.grid(True)

# Add a zoomed-in inset
axins = plt.axes([0.6, 0.6, 0.25, 0.25])
axins.plot(time, samples)
axins.set_xlim(0, 0.1)  # Zoom to first 0.1 seconds
axins.set_ylim(-1.1, 1.1)
axins.grid(True)

plt.tight_layout()
plt.show()
