import { DashboardNotification } from "@/utils/types";

type NotificationCenterProps = {
  notifications: DashboardNotification[];
  dismissNotification: (id: string) => void;
};

export function NotificationCenter({
  notifications,
  dismissNotification,
}: NotificationCenterProps) {
  return (
    <div className="td-notification-stack" aria-live="polite" aria-atomic="false">
      {notifications.map((notification) => (
        <div
          key={notification.id}
          className={`td-notification td-notification-${notification.tone}`}
          role="status"
        >
          <div className="td-notification-body">
            <div className="td-notification-title">{notification.title}</div>
            {notification.message ? (
              <div className="td-notification-message">{notification.message}</div>
            ) : null}
          </div>
          <button
            type="button"
            className="td-notification-close"
            aria-label="Dismiss notification"
            onClick={() => dismissNotification(notification.id)}
          >
            ×
          </button>
        </div>
      ))}
    </div>
  );
}

export default NotificationCenter;
