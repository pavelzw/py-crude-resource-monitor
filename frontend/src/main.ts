import uPlot from "uplot";
import "uplot/dist/uPlot.min.css";
import {
  completeReportToSeries,
  parseJsonProcessReport,
  ProcessReport,
  SAMPLE_REPORT,
} from "./data";
import "./style.css";
import { binarySearch } from "./util";

const report = parseJsonProcessReport("2432", SAMPLE_REPORT);
const completeReport = [report];

const uplotData = completeReportToSeries(completeReport);

function buildMemorySeries(report: ProcessReport): uPlot.Series {
  return {
    show: true,

    spanGaps: false,
    scale: "byte",

    // in-legend display
    label: `RAM (${report.id})`,
    value: (_, rawValue) =>
      rawValue === null ? "--" : (rawValue / 1024 / 1024).toFixed(1) + "MiB",

    // series style
    stroke: "red",
    width: 1,
    dash: [5],
  };
}

function buildCpuSeries(report: ProcessReport): uPlot.Series {
  return {
    show: true,

    spanGaps: false,
    scale: "%",

    // in-legend display
    label: `CPU (${report.id})`,
    value: (_, rawValue) =>
      rawValue === null ? "--" : rawValue.toFixed(0) + "%",

    // series style
    stroke: "blue",
    width: 1,
  };
}

function displayStackframePlugin(reports: ProcessReport[]): uPlot.Plugin {
  let dataIdx: null | number = null;
  let seriesIdx: null | number = null;

  function updateDisplay() {
    if (dataIdx === null || seriesIdx === null) {
      return;
    }
    // -1 for x?
    const reportIndex = Math.floor((seriesIdx - 1) / 2);
    const report = reports[reportIndex];

    const index = binarySearch(
      report.entries,
      uplotData.xData[dataIdx],
      (it) => it.time
    );
    if (index < 0) {
      return;
    }
    const threads = report.entries[index].stacktraces;
    const traceArea = document.getElementById("stacktraceArea")!;

    let content = "";
    for (const thread of threads) {
      content += `<h3>${thread.os_thread_id} (${thread.thread_name})</h3>`;
      content += `<div class="frames">`;
      let frames = "";
      for (const frame of thread.frames) {
        frames += `${frame.name} (${frame.short_filename}:${frame.line})\n`;
      }
      const span = document.createElement("span");
      span.textContent = frames;
      content += span.innerHTML;
      content += "</div>";
    }

    traceArea.innerHTML = content;
  }

  return {
    hooks: {
      setCursor: (u) => {
        const idx = u.cursor.idx;
        if (idx !== dataIdx) {
          dataIdx = idx || null;
          updateDisplay();
        }
      },
      setSeries(_, idx) {
        if (seriesIdx !== idx) {
          seriesIdx = idx;
          updateDisplay();
        }
      },
    },
  };
}

const opts: uPlot.Options = {
  title: "My Chart",
  id: "chart1",
  class: "my-chart",
  width: 800,
  height: 600,
  series: [{}],
  legend: {
    live: true,
  },
  scales: {
    byte: {
      distr: 1,
    },
  },
  axes: [
    {},
    {
      size: 80,
      scale: "byte",
      values: (_, ticks) =>
        ticks.map(
          (rawValue) => (rawValue / 1024 / 1024 / 1024).toFixed(1) + "GiB"
        ),
    },
    {
      scale: "%",
      values: (_, ticks) => ticks.map((rawValue) => rawValue.toFixed(0) + "%"),
      side: 1,
      grid: { show: false },
    },
  ],
  cursor: {
    focus: {
      prox: 5,
    },
    lock: true,
  },
  plugins: [displayStackframePlugin(uplotData.reports)],
};

for (const report of uplotData.reports) {
  opts.series.push(buildCpuSeries(report));
  opts.series.push(buildMemorySeries(report));
}

let uplot = new uPlot(
  opts,
  [uplotData.xData.map((it) => it / 1000), ...uplotData.yData],
  document.getElementById("chartContainer")!
);

const observer = new ResizeObserver(() => {
  const container = document.getElementById("chartContainer")!;
  uplot.setSize(container.getBoundingClientRect());
});
observer.observe(document.getElementById("chartContainer")!);
