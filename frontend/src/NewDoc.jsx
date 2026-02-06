import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import * as api from "./api";

export default function NewDoc() {
  const navigate = useNavigate();
  const [error, setError] = useState(null);

  useEffect(() => {
    let cancelled = false;
    api
      .createDoc()
      .then((res) => {
        if (!cancelled) navigate(`/d/${res.id}`, { replace: true });
      })
      .catch((e) => {
        if (!cancelled) setError("Failed to create document: " + e.message);
      });
    return () => {
      cancelled = true;
    };
  }, [navigate]);

  if (error) {
    return (
      <div className="loading">
        <p>{error}</p>
      </div>
    );
  }

  return (
    <div className="loading">
      <div className="loading-spinner" />
      <p>Growing a new grove...</p>
    </div>
  );
}
