//
// scripts.js
//
// Main script file for generating the graphs and importing
// the database values, also have the different functions
// for changing the timestamps.
//
// Author: Tiemen Slotboom <tiemen.slotboom@gmail.com>
// Date: December 6th, 2019
// Last Edit: January 9th, 2020
//


//3 months: Array(12961)
//1 month: Array(4321)
//1 week: Array(1009)
//1 day: Array(145)
//default timestamp:
array_length = 12961;

if(options.timeframe == "Week") {
	array_length = 1009;
} else if(options.timeframe == "Month") {
	array_length = 4321;
}

//global variables so all functions can use the same data and graph
var graph;
var temp_parsedResponse;
var temp_unpackData;

unit_dictionary = {
    temperature: {
        "Kelvin": "K",
        "Celsius": "℃",
        "Fahrenheit": "℉",
    },
    pressure: {
        "Atmosphere": "atm",
        "Millibar": "mbar",
        "Bar": "bar",
        "PSI": "PSI",
        "Mercury": "Hg",
    }
}

//the first thing the website does is fetching the data from the api
//and set it in the graph with the default timestamp
const loadData = () => {
  fetch('https://mppadding.dev:3010/api/v1/usage?temperature=' + options.temperature + "&pressure=" + options.pressure)
    .then( response => {
      if (response.status !== 200) {
        console.log(response);
      }
      return response;
    })
    .then(response => response.json())
    .then(parsedResponse => {
      const unpackData = (array, key) => {
        return array.map(obj => Object.assign({}, { x: Date.parse(obj['time']), y: obj[key] }))
      };

      console.log(parsedResponse.length);
      console.log(parsedResponse.slice(parsedResponse.length - array_length,parsedResponse.length));
      temp_parsedResponse = parsedResponse;
      parsedResponse = parsedResponse.slice(parsedResponse.length - array_length,parsedResponse.length);
      temp_unpackData = unpackData;

      const palette = new Rickshaw.Color.Palette({ scheme: 'colorwheel' }); //Auto generating the palette for the colors of the different graphs
      graph = new Rickshaw.Graph({ //making the chart
        element: document.querySelector('#chart'), //the chart id for in the html file
        stack: false,
        renderer: 'line', //this is the type of graph
        series: [
          { //data of humidity
            name: 'Humidity',
            data: unpackData(parsedResponse, 'humidity'), 
            color: palette.color()
          },
          { //data of lux
            name: 'Lux',
            data: unpackData(parsedResponse, 'lux'),
            color: palette.color()
          },
          { //data of temperature
            name: 'Temperature',
            data: unpackData(parsedResponse, 'temperature'),
            color: palette.color()
          },
          { //data of pressure
            name: 'Pressure',
            data: unpackData(parsedResponse, 'pressure'),
            color: palette.color()
          }
        ]
      });

      addContents();
      return graph.render();
    })
    .catch( error => console.log(error) );

}

//Function that adds the contents to the graph
function addContents(){

  graph.configure({
    width: window.innerWidth * 0.6,
    height: window.innerHeight * 0.5
  });

  //The hover event so the data can be easily seen
  var hoverDetail = new Rickshaw.Graph.HoverDetail( {
    graph: graph,
    xFormatter: function(x) {
      const options = { weekday: 'long', year: 'numeric', month: 'long', day: 'numeric' };
      return new Date(x).toLocaleDateString('nl-NL', options) + " " + new Date(x).toLocaleTimeString();
    },
    formatter : function(series, x, y){

      switch(series.name){
        case "Humidity":
          return series.name + ": " + Math.round(y*100)/100 + "%";
          break;
        case "Lux":
          return series.name + ": " + Math.round(y*100)/100 + "%";
          break;
        case "Temperature":
          return series.name + ": " + Math.round(y*100)/100 + " " + unit_dictionary.temperature[options.temperature];
          break;
        case "Pressure":
          return series.name + ": " + Math.round(y*1000)/1000 + " " + unit_dictionary.pressure[options.pressure];
          break;
      }
      return "ERROR: OOF";
    }
  });

  //the X axis generated with a function for the time
  var xAxis = new Rickshaw.Graph.Axis.X({
    graph: graph,
    tickFormat: function(x){
      return new Date(x).toLocaleTimeString();
    }
  });

  //the Y axis for displaying the values
  const yAxis = new Rickshaw.Graph.Axis.Y({
    element: document.getElementById('y-axis'),
    graph: graph,
    orientation: 'left',
    tickFormat: Rickshaw.Fixtures.Number.formatKMBT,
  });

  //The legend for displaying the graphs
  var myLegend = new Rickshaw.Graph.Legend({
    graph: graph,
    element: document.querySelector("#mylegend")
  });

  //Selecting which graph should be displayed
  var toggling = new Rickshaw.Graph.Behavior.Series.Toggle({
    graph: graph,
    legend: myLegend
  });

  //For hovering over the legend to see which graph it is
  var highlighter = new Rickshaw.Graph.Behavior.Series.Highlight({
    graph: graph,
    legend: myLegend
  });

  //The previewSlider for underneath the chart for selecting the range
  var previewSlider = new Rickshaw.Graph.RangeSlider.Preview({
    graph: graph,
    element: document.querySelector("#previewSlider")
  });

  //updating the cirkels with the latest values
  document.getElementById('temperatureValue').innerHTML = Math.round(temp_unpackData(temp_parsedResponse, 'temperature')[temp_parsedResponse.length-1].y * 100)/100 + " " + unit_dictionary.temperature[options.temperature];
  document.getElementById('humidityValue').innerHTML = Math.round(temp_unpackData(temp_parsedResponse, 'humidity')[temp_parsedResponse.length-1].y * 100)/100 + "%";
  document.getElementById('pressureValue').innerHTML = Math.round(temp_unpackData(temp_parsedResponse, 'pressure')[temp_parsedResponse.length-1].y * 1000)/1000 + " " + unit_dictionary.pressure[options.pressure];
  document.getElementById('luminosityValue').innerHTML = Math.round(temp_unpackData(temp_parsedResponse, 'lux')[temp_parsedResponse.length-1].y * 100)/100 + "%";

}

document.addEventListener('DOMContentLoaded', loadData);

//This function handles the updates, this function is called from the different buttons
//in the html file, this is where the timestamp is being changed
function updateGraph(timestamp){

  switch(timestamp){
    case "QuarterYear":
      array_length = 12961;
      break;
    case "Month":
      array_length = 4321;
      break;
    case "Week":
      array_length = 1009;
      break;
    case "Day":
      array_length = 145;
      break;
    case "last":
      array_length = array_length;
      break;
  }

  clearGraph();
  parsedResponse = temp_parsedResponse.slice(temp_parsedResponse.length - array_length,temp_parsedResponse.length);
  const palette = new Rickshaw.Color.Palette({ scheme: 'colorwheel' }); //Auto generating the palette for the colors of the different graphs
  graph = new Rickshaw.Graph({ //making the chart
    element: document.querySelector('#chart'), //the chart id for in the html file
    //width: 1000,
    //height: 400,
    stack: false,
    renderer: 'line', //this is the type of graph
    series: [
      { //data of humidity
        name: 'Humidity',
        data: temp_unpackData(parsedResponse, 'humidity'), 
        color: palette.color()
      },
      { //data of lux
        name: 'Lux',
        data: temp_unpackData(parsedResponse, 'lux'),
        color: palette.color()
      },
      { //data of temperature
        name: 'Temperature',
        data: temp_unpackData(parsedResponse, 'temperature'),
        color: palette.color()
      },
      { //data of pressure
        name: 'Pressure',
        data: temp_unpackData(parsedResponse, 'pressure'),
        color: palette.color()
      }
    ]
  });

  addContents();

  graph.render();

}

function customTimestamp(){

  var person = prompt("Please enter the amount of hours: ", "");
  hours = parseFloat(person);

  if (Number.isNaN(hours)) {
    if(person == null || person == ""){
      return;
    }else{
      alert("Only use numbers!");
    }
  } else if(hours <= 0 || hours >= 2160) {
    alert("Hours can't be less or equal to 0 or higher than 2160!");
  }else {
    array_length = hours*6;
    updateGraph('last');
  }

}

//To update the graph, first of all the old graph needs to be deleted
//that is what this function does
function clearGraph() {
  $('#mylegend').empty();
  $('#y-axis').html(
    '<div id="chart"></div><div id="timeline"></div><div id="slider"></div>'
  );
  $('#previewSlider').html(
    ''
  );
  $('#chart').html(
    '<div id="chart"></div><div id="timeline"></div><div id="slider"></div>'
  );
}
