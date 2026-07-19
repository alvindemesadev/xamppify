import { useEffect, useState } from "react";
import { listCertificates, readCertificate } from "@/lib/ipc";

export function useCertExpiry() {
  const [expiringCount, setExpiringCount] = useState(0);

  useEffect(() => {
    (async () => {
      try {
        const certFiles = await listCertificates();
        const crts = certFiles.filter((c) => !c.is_key);
        let count = 0;
        for (const c of crts.slice(0, 10)) {
          const info = await readCertificate(c.path);
          if (info.valid_to) {
            const parsed = Date.parse(info.valid_to);
            if (parsed && parsed - Date.now() < 30 * 24 * 60 * 60 * 1000) {
              count++;
            }
          }
        }
        setExpiringCount(count);
      } catch {
        // ignore
      }
    })();
  }, []);

  return expiringCount;
}
