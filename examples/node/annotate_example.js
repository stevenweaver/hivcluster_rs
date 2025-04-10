const fs = require('fs');
const path = require('path');
const hivannotate = require('../../pkg/hivannotate-node/hivcluster_rs.js');

// Example usage of HIVAnnotate
async function main() {
  console.log('HIVAnnotate Example');
  
  try {
    // Read network JSON file
    const networkPath = path.resolve(__dirname, '../../test_network.json');
    const networkJson = fs.readFileSync(networkPath, 'utf8');
    
    // Read attributes JSON file
    const attributesPath = path.resolve(__dirname, '../../32190349_attributes.json');
    const attributesJson = fs.readFileSync(attributesPath, 'utf8');
    
    // Read schema JSON file
    const schemaPath = path.resolve(__dirname, '../../32190349_schema.json');
    const schemaJson = fs.readFileSync(schemaPath, 'utf8');
    
    console.log('Annotating network with patient attributes...');
    
    // Check available functions
    console.log('Available functions in hivannotate:');
    console.log(Object.keys(hivannotate));
    
    // Annotate the network
    let annotatedJson;
    try {
      annotatedJson = hivannotate.annotate_network_json(networkJson, attributesJson, schemaJson);
      console.log('Annotation successful!');
      
      // Write the result to a file
      const outputPath = path.resolve(__dirname, '../../annotated_results.json');
      fs.writeFileSync(outputPath, annotatedJson);
      
      console.log(`Annotation complete! Result saved to ${outputPath}`);
    } catch (error) {
      console.error('Error during annotation:', error);
      return;
    }
    
    // Print some statistics
    const result = JSON.parse(annotatedJson);
    const hasTraceResults = result.trace_results !== undefined;
    const rootObj = hasTraceResults ? result.trace_results : result;
    
    console.log('\nAnnotation Statistics:');
    console.log(`- Network contains ${rootObj.Nodes.length} nodes`);
    
    // Count nodes with annotations
    const nodesWithAttributes = rootObj.Nodes.filter(n => n.patient_attributes !== undefined).length;
    console.log(`- ${nodesWithAttributes} nodes annotated with patient attributes`);
    
    // Show schema fields
    console.log('\nAttribute Schema:');
    const schemaFields = Object.keys(rootObj.patient_attribute_schema || {});
    schemaFields.forEach(field => {
      console.log(`- ${field}`);
    });
    
  } catch (error) {
    console.error('Error:', error.message);
  }
}

main();