import "uplot/dist/uPlot.min.css";
import {
  CompleteReport,
  completeReportToSeries,
  fetchReportByName,
  fetchReportNames,
} from "./data";
import { buildPlot } from "./plot";
import "./style.css";

(async () => {
  const stackTraceArea = document.getElementById("stacktraceArea")!;

  stackTraceArea.textContent = "Fetching report names..."
  const reportNames = await fetchReportNames();

  const reports = [];
  for (const name of reportNames) {
    stackTraceArea.textContent += `\nFetching report ${name}`
    reports.push(await fetchReportByName(name));
  }

  stackTraceArea.textContent += "\n\nBuilding Graph series"
  const uplotData = completeReportToSeries(reports as CompleteReport);

  stackTraceArea.textContent += "\n\nDisplaying plot"
  buildPlot(uplotData);
})();
