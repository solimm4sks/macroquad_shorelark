use rand::Rng;
#[cfg(test)]
use rand::SeedableRng;
#[cfg(test)]
use rand_chacha::ChaCha8Rng;

#[derive(Debug)]
pub struct Network{
    layers: Vec<Layer>,
}

pub struct LayerTopology {
    pub neurons: usize,
}

#[derive(Debug)]
struct Layer{
    neurons: Vec<Neuron>,
}

#[derive(Debug)]
struct Neuron{
    bias: f32,
    weights: Vec<f32>,
}

impl Network {
    pub fn new() -> Self{
        todo!()
    }

    pub fn propagate(&self, mut inputs: Vec<f32>) -> Vec<f32> {
        for layer in &self.layers {
            inputs = layer.propagate(inputs);
        }
        return inputs;
    }

    pub fn random(rng: &mut dyn rand::RngCore, layers: &[LayerTopology]) -> Self {
        assert!(layers.len() > 1);

        let mut built_layers = Vec::new();

        for adj_layers in layers.windows(2) {
            let input_neurons: usize = adj_layers[0].neurons;
            let output_neurons: usize = adj_layers[1].neurons;

            built_layers.push(Layer::random(rng, input_neurons, output_neurons));
        }

        Self { layers: built_layers }
    }

    pub fn weights(&self) -> Vec<f32>{
        let mut weights = Vec::new();
        for layer in &self.layers {
            for neuron in &layer.neurons {
                weights.push(neuron.bias);
    
                for weight in &neuron.weights {
                    weights.push(*weight);
                }
            }
        }
        weights
    }

    pub fn from_weights_inplace(&mut self, layers_topology: &[LayerTopology], weights: impl IntoIterator<Item = f32>){
        self.layers = Network::from_weights(layers_topology, weights).layers;
    }

    pub fn from_weights(layers: &[LayerTopology], weights: impl IntoIterator<Item = f32>) -> Self {
        assert!(layers.len() > 1);

        let mut weights = weights.into_iter();
        let layers = layers.windows(2).map(|layers| {
                Layer::from_weights(
                    layers[0].neurons,
                    layers[1].neurons,
                    &mut weights,
                )
            }).collect();

        if weights.next().is_some() {
            panic!("got too many weights");
        }

        Self { layers }
    }
}

impl Layer{
    fn propagate(&self, inputs: Vec<f32>) -> Vec<f32>{
        let mut outputs = Vec::with_capacity(self.neurons.len());
        for neuron in &self.neurons {
            let output = neuron.propagate(&inputs);
            outputs.push(output);
        }

        outputs
    }

    pub fn random(rng: &mut dyn rand::RngCore, input_neurons: usize, output_neurons: usize) -> Self {
        let mut neurons = Vec::with_capacity(output_neurons);
        for _ in 0..output_neurons {
            neurons.push(Neuron::random(rng, input_neurons));
        }

        Self { neurons }
    }

    pub fn from_weights(input_size: usize, output_size: usize, weights: &mut dyn Iterator<Item = f32>) -> Self {
        let neurons = (0..output_size).map(|_| Neuron::from_weights(input_size, weights)).collect();
        Self { neurons }
    }
}

impl Neuron{
    fn propagate(&self, inputs: &[f32]) -> f32{
        assert_eq!(inputs.len(), self.weights.len());

        let mut output: f32 = 0.0;
        for (&input, &weight) in inputs.iter().zip(&self.weights) { //faster than without zipping since outofbounds checking
            output += input * weight;
        }
        output += self.bias;

        return output.max(0.0);
    }

    pub fn random(rng: &mut dyn rand::RngCore, output_size: usize) -> Self {
        let bias = rng.gen_range(-1.0..=1.0);

        let weights = (0..output_size)
            .map(|_| rng.gen_range(-1.0..=1.0))
            .collect();

        Self { bias, weights }
    }

    pub fn from_weights(output_neurons: usize, weights: &mut dyn Iterator<Item = f32>) -> Self {
        let bias = weights.next().expect("got not enough weights");

        let weights = (0..output_neurons)
            .map(|_| weights.next().expect("got not enough weights"))
            .collect();

        Self { bias, weights }
    }
}

#[test]
fn test() {
    // Because we always use the same seed, our `rng` in here will
    // always return the same set of values
    let mut rng = ChaCha8Rng::from_seed(Default::default());
    let neuron = Neuron::random(&mut rng, 4);

    assert_eq!(neuron.bias, -0.6255188);
    assert_eq!(neuron.weights, &[0.67383957, 0.8181262, 0.26284897, 0.5238807]);
}

#[test]
fn test2() {
    let neuron = Neuron {
        bias: 0.5,
        weights: vec![-0.3, 0.8],
    };

    // Ensures `.max()` (our ReLU) works:
    approx::assert_relative_eq!(
        neuron.propagate(&[-10.0, -10.0]),
        0.0,
    );

    // `0.5` and `1.0` chosen by a fair dice roll:
    approx::assert_relative_eq!(
        neuron.propagate(&[0.5, 1.0]),
        (-0.3 * 0.5) + (0.8 * 1.0) + 0.5,
    );

    // We could've written `1.15` right away, but showing the entire
    // formula makes our intentions clearer
}