import { useCallback } from "react";
import { useCatalogStore } from "../../stores/catalogStore";
import { SearchInput } from "../common/SearchInput";
import styles from "./FilterBar.module.css";

export function FilterBar() {
  const { filter, setFilter, searchImages, clearFilter } = useCatalogStore();
  const handleSearch = useCallback((query: string) => { setFilter({ query }); searchImages(); }, [setFilter, searchImages]);

  return (
    <div className={styles.bar}>
      <SearchInput value={filter.query} onChange={handleSearch} placeholder="Search photos..." />
      <div className={styles.filters}>
        <select className={styles.select} value={filter.ratingMin} onChange={(e) => { setFilter({ ratingMin: Number(e.target.value) }); searchImages(); }}>
          <option value={0}>All Ratings</option>
          <option value={1}>1+ Stars</option><option value={2}>2+ Stars</option><option value={3}>3+ Stars</option><option value={4}>4+ Stars</option><option value={5}>5 Stars</option>
        </select>
        <select className={styles.select} value={filter.flag ?? ""} onChange={(e) => { setFilter({ flag: (e.target.value || null) as any }); searchImages(); }}>
          <option value="">All Flags</option><option value="picked">Picked</option><option value="rejected">Rejected</option>
        </select>
        <select className={styles.select} value={filter.colorLabel ?? ""} onChange={(e) => { setFilter({ colorLabel: (e.target.value || null) as any }); searchImages(); }}>
          <option value="">All Colors</option><option value="red">Red</option><option value="yellow">Yellow</option><option value="green">Green</option><option value="blue">Blue</option><option value="purple">Purple</option>
        </select>
        {(filter.query || filter.ratingMin > 0 || filter.flag || filter.colorLabel) && <button className={styles.clearBtn} onClick={clearFilter}>Clear</button>}
      </div>
    </div>
  );
}
