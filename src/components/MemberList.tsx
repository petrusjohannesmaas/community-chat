interface Props {
  members: string[];
  currentUser: string;
  activeConversation: string;
  onSelectGlobal: () => void;
  onSelectMember: (username: string) => void;
}

export default function MemberList({
  members,
  currentUser,
  activeConversation,
  onSelectGlobal,
  onSelectMember,
}: Props) {
  const sorted = [...members].sort((a, b) =>
    a === currentUser ? -1 : b === currentUser ? 1 : a.localeCompare(b)
  );

  return (
    <div className="sidebar">
      <div className="sidebar-header">Community Chat</div>
      <div className="sidebar-nav">
        <button
          className={`sidebar-btn${activeConversation === "global" ? " active" : ""}`}
          onClick={onSelectGlobal}
        >
          # Global Chat
        </button>
      </div>
      <div className="member-list">
        {sorted.map((member) => (
          <div
            key={member}
            className={`member-item${activeConversation === `dm:${member}` ? " active" : ""}`}
            onClick={() => onSelectMember(member)}
          >
            <span className="member-dot" />
            <span className="member-name">
              {member === currentUser ? `${member} (you)` : member}
            </span>
          </div>
        ))}
      </div>
    </div>
  );
}
