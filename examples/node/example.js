const fs = require('fs');
const path = require('path');
const { build_network } = require('../../pkg/node');

// Define a sample CSV or read from a file
const sampleCSV = `ID1,ID2,0.01
ID1,ID3,0.02
ID2,ID4,0.015
ID3,ID4,0.01
ID5,ID6,0.025
ID7,ID8,0.01`;

// Distance threshold
const threshold = 0.03;

try {
    console.log('Building network...');
    const network = build_network(sampleCSV, threshold);
    
    // Output network statistics
    console.log('\nNetwork Statistics:');
    console.log(`Nodes: ${network.nodes.length}`);
    console.log(`Edges: ${network.edges.length}`);
    console.log(`Clusters: ${network.clusters.length}`);
    
    // Find largest cluster
    const largestCluster = network.clusters.reduce((max, cluster) => 
        cluster.size > max ? cluster.size : max, 0);
    console.log(`Largest Cluster Size: ${largestCluster}`);
    
    // Display cluster information
    console.log('\nClusters:');
    network.clusters.forEach(cluster => {
        console.log(`Cluster ${cluster.id}: ${cluster.size} nodes - ${cluster.nodes.join(', ')}`);
    });
    
    // Save the JSON output to a file
    fs.writeFileSync(
        path.join(__dirname, 'network_output.json'), 
        JSON.stringify(network, null, 2)
    );
    console.log('\nNetwork JSON saved to network_output.json');
    
} catch (error) {
    console.error('Error processing network:', error);
}