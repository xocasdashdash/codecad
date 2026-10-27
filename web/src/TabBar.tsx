interface Tab {
  id: string;
  label: string;
}

interface Props {
  tabs: Tab[];
  activeId: string | null;
  onSelect: (id: string) => void;
  onClose: (id: string) => void;
  onAdd: () => void;
  onCompare: () => void;
  comparing: boolean;
}

export function TabBar({ tabs, activeId, onSelect, onClose, onAdd, onCompare, comparing }: Props) {
  return (
    <div className="tab-bar">
      {tabs.map((tab) => (
        <div
          key={tab.id}
          className={`tab${tab.id === activeId ? " active" : ""}`}
          onClick={() => onSelect(tab.id)}
        >
          <span className="tab-label">{tab.label}</span>
          <button
            className="tab-close"
            onClick={(e) => {
              e.stopPropagation();
              onClose(tab.id);
            }}
          >
            x
          </button>
        </div>
      ))}
      <button className="tab-add" onClick={onAdd}>
        +
      </button>
      <button className={`tab-compare${comparing ? " active" : ""}`}
        onClick={onCompare} title="Show two drawings side by side">
        {comparing ? "Single view" : "Compare"}
      </button>
    </div>
  );
}
