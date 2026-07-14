import { useState, useEffect } from "react";
import { api, onShortcutRecorded } from "../../lib/ipc";
import { HeatLegend } from "../ui/HeatLegend";

interface KeyProps {
  label: string;
  widthClass?: string;
  count: number;
}

const Key: React.FC<KeyProps> = ({ label, widthClass = "kb-w-1", count }) => {
  // Determine heat level 0-5
  let level = 0;
  if (count > 0) {
    if (count <= 5) level = 1;
    else if (count <= 20) level = 2;
    else if (count <= 50) level = 3;
    else if (count <= 150) level = 4;
    else level = 5;
  }

  return (
    <div className={`kb-key ${widthClass} h${level}`} title={`${label}: ${count} presses`}>
      {label}
    </div>
  );
};

export const Heatmap: React.FC = () => {
  const [data, setData] = useState<Record<string, number>>({});
  const [appFilter, setAppFilter] = useState("All Apps");
  const [days, setDays] = useState(7);
  const [apps, setApps] = useState<string[]>([]);

  const fetchData = async () => {
    try {
      const [heatmap, appList] = await Promise.all([
        api.getHeatmapData(appFilter, days),
        api.getAppsList()
      ]);
      setData(heatmap);
      setApps(appList);
    } catch (err) {
      console.error("Heatmap fetch error:", err);
    }
  };

  useEffect(() => {
    fetchData();
    const unlisten = onShortcutRecorded(() => fetchData());
    return () => { unlisten.then(fn => fn()); };
  }, [appFilter, days]);

  const getCount = (key: string) => data[key] || 0;

  return (
    <div className="heatmap-tab">
      <div className="heatmap-controls">
        <div className="control-group">
          <label className="label-mono">App Filter</label>
          <select value={appFilter} onChange={(e) => setAppFilter(e.target.value)} className="custom-select">
            {apps.map(app => <option key={app} value={app}>{app}</option>)}
          </select>
        </div>
        <div className="control-group">
          <label className="label-mono">Time Range</label>
          <div className="segmented-control">
            {[1, 7, 30, 0].map(d => (
              <button 
                key={d} 
                className={days === d ? "active" : ""} 
                onClick={() => setDays(d)}
              >
                {d === 0 ? "All" : d === 1 ? "Today" : `${d}d`}
              </button>
            ))}
          </div>
        </div>
      </div>

      <div className="keyboard-container">
        {/* Row 1 */}
        <div className="kb-row">
          {["`","1","2","3","4","5","6","7","8","9","0","-","="].map(k => <Key key={k} label={k} count={getCount(k)} />)}
          <Key label="Backspace" widthClass="kb-w-20" count={getCount("Backspace")} />
        </div>
        {/* Row 2 */}
        <div className="kb-row">
          <Key label="Tab" widthClass="kb-w-15" count={getCount("Tab")} />
          {["Q","W","E","R","T","Y","U","I","O","P","[","]","\\"].map(k => <Key key={k} label={k} count={getCount(k)} />)}
        </div>
        {/* Row 3 */}
        <div className="kb-row">
          <Key label="Caps" widthClass="kb-w-20" count={getCount("CapsLock")} />
          {["A","S","D","F","G","H","J","K","L",";","'"].map(k => <Key key={k} label={k} count={getCount(k)} />)}
          <Key label="Enter" widthClass="kb-w-20" count={getCount("Enter")} />
        </div>
        {/* Row 4 */}
        <div className="kb-row">
          <Key label="Shift" widthClass="kb-w-25" count={getCount("Shift")} />
          {["Z","X","C","V","B","N","M",",",".","/"].map(k => <Key key={k} label={k} count={getCount(k)} />)}
          <Key label="Shift" widthClass="kb-w-25" count={getCount("Shift")} />
        </div>
        {/* Row 5 */}
        <div className="kb-row">
          <Key label="Ctrl" widthClass="kb-w-15" count={getCount("Ctrl")} />
          <Key label="Win" widthClass="kb-w-15" count={getCount("Win")} />
          <Key label="Alt" widthClass="kb-w-15" count={getCount("Alt")} />
          <Key label="Space" widthClass="kb-w-sp" count={getCount("Space")} />
          <Key label="Alt" widthClass="kb-w-15" count={getCount("Alt")} />
          <Key label="Win" widthClass="kb-w-15" count={getCount("Win")} />
          <Key label="Menu" widthClass="kb-w-15" count={getCount("Menu")} />
          <Key label="Ctrl" widthClass="kb-w-15" count={getCount("Ctrl")} />
        </div>
      </div>

      <HeatLegend />
    </div>
  );
};
