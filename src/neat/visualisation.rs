use super::structs::genome::Genome;
use super::structs::problem::Problem;
use std::{
    fs::File,
    io::Write,
};

const HEADER: &'static str = "
<!DOCTYPE html>
<html lang=\"en\">
<head>
    <meta charset=\"UTF-8\">
    <meta http-equiv=\"X-UA-Compatible\" content=\"IE=edge\">
    <script src=\"https://cdnjs.cloudflare.com/ajax/libs/cytoscape/3.21.2/cytoscape.min.js\" integrity=\"sha512-I1SEbsEt/GKuL+DudstOv/Ain+H/7IO3J2VHprg9IQOIIRmNQTC2WMdPXrHmYF9Qo1jRGnMrHtY7NqP9oaKFAQ==\" crossorigin=\"anonymous\" referrerpolicy=\"no-referrer\"></script>
    <title>Document</title>
</head>
<body>
<div id=\"cy\" style=\"width: 800px;
height: 600px;
display: block;\"></div>
  <script>
  var cy = cytoscape({

container: document.getElementById('cy'), // container to render in

elements: [ // list of graph elements to start with
";

const FOOTER: &'static str = "
],

layout: {
  name: 'preset'
},

style: [ // the stylesheet for the graph
  {
    selector: 'node',
    style: {
      'background-color': '#666',
      'label': 'data(id)'
    }
  },

  {
    selector: 'edge',
    style: {
      'width': 3,
      'line-color': '#ccc',
      'target-arrow-color': '#ccc',
      'target-arrow-shape': 'triangle',
      'curve-style': 'bezier'
    }
  }
],

});
  </script>
</body>
</html>
";

pub fn visualise(genome:&Genome, problem:&Problem) {
    let mut file = File::create("out.html").unwrap();
    file.write_all(HEADER.as_bytes()).unwrap();

    for n in 0..genome.nodes {
        if n < problem.inputs {
          write!(&mut file, "
          {{ 
            data: {{ 
              id: '{0}', 
               
          }}, position: {{ 
            y: {0}*100,
            x: -400 
          }},
          style: {{ 
            'label': '[in {0}]'
          }}
          }},\n", n).unwrap();
        }
        else if n >= problem.inputs && n < problem.inputs + problem.outputs {
          write!(&mut file, "
          {{ 
            data: {{ 
              id: '{0}'
          }}, 
          position: {{  
          y: {1}*100,
          x: 400 
        }}, 
          style: {{ 
            'label': '[out {0}]'
          }}
          }},\n", n, n - problem.inputs ).unwrap();
        }
        else {
          write!(&mut file, "
          {{ 
            data: {{ 
              id: '{0}'
          }}, 
          position: {{ 
            y: {1}*100,
            x: 0 
          }}, 
          style: {{ 
            'label': '[{0}]'
          }}
          }},\n", n, n - (problem.inputs + problem.outputs) + 1).unwrap();
        }
    }
    for gene in genome.genes.iter() {
        if gene.enabled {
          write!(&mut file, "
          {{ data: 
            {{ id: '{input}-{output}', source: '{input}', target: '{output}'}},
            style: {{
              'label': '{weight:.2}'
            }} 
          }},\n", input=gene.input, output=gene.output, weight=gene.weight).unwrap();
        }
    }
    
    file.write_all(FOOTER.as_bytes()).unwrap();
}