import "uplot/dist/uPlot.min.css";
import {
  completeReportToSeries,
  parseJsonProcessReport,
  SAMPLE_REPORT,
} from "./data";
import { buildPlot } from "./plot";
import "./style.css";

const report = parseJsonProcessReport("2432", SAMPLE_REPORT);
const completeReport = [report];

const uplotData = completeReportToSeries(completeReport);

let uplot = buildPlot(uplotData);

const observer = new ResizeObserver(() => {
  const container = document.getElementById("chartContainer")!;
  uplot.setSize(container.getBoundingClientRect());
});
observer.observe(document.getElementById("chartContainer")!);
