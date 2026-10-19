import pandas as pd
import matplotlib.pyplot as plt
import matplotlib.ticker as ticker
import seaborn as sns
from pathlib import Path

plots_dir_path = Path("plots")
plots_dir_path.mkdir(exist_ok=True)

sns.set_theme(style="whitegrid")

benchmarks = [
    ("latency_benchmark_results.csv", "latency"),
    ("throughput_benchmark_results.csv", "throughput"),
    ("through_old_benchmark_results.csv", "throughput_old"),
    ("pipeline_benchmark_results.csv", "pipeline")
]

for csv_file, prefix in benchmarks:
    df = pd.read_csv(csv_file)
    
    param_col = "k" if "k" in df.columns else ("threads" if "threads" in df.columns else None)

    for file_name, group in df.groupby("file"):
        filename_clean = file_name.replace("/", "_")
        has_param = param_col is not None and param_col in group.columns

        plt.figure(figsize=(10, 6))
        ax1 = sns.lineplot(
            data=group, 
            x="block_size", 
            y="time_seconds", 
            hue=param_col if has_param else None, 
            marker="o", 
            palette="tab10" if has_param else None,
            linewidth=2
        )
        plt.xscale("log", base=2)
        
        ax1.yaxis.set_major_formatter(ticker.FormatStrFormatter('%.3f'))
        
        plt.title(f"[{prefix.upper()}] Czas wykonania w zależności od rozmiaru bloku ({file_name})", fontsize=14, pad=15)
        plt.xlabel("Rozmiar bloku w bajtach", fontsize=12)
        plt.ylabel("Czas wykonania (s)", fontsize=12)
        
        if has_param:
            legend_title = "Liczba wątków" if param_col == "threads" else "Parametr K"
            plt.legend(title=legend_title, bbox_to_anchor=(1.05, 1), loc="upper left")
            
        plt.tight_layout()
        
        output_path_1 = plots_dir_path / f"{prefix}_{filename_clean}_time_vs_block_size.png"
        plt.savefig(output_path_1, dpi=300)
        plt.close()
        print(f"Zapisano wykres: {output_path_1}")

        if has_param:
            plt.figure(figsize=(10, 6))
            ax2 = sns.lineplot(
                data=group, 
                x=param_col, 
                y="time_seconds", 
                hue="block_size", 
                marker="s", 
                palette="tab10",
                linewidth=2
            )
            plt.xscale("log", base=2)
            
            ax2.yaxis.set_major_formatter(ticker.FormatStrFormatter('%.3f'))
            
            x_label_text = "Liczba wątków (skala logarytmiczna)" if param_col == "threads" else "Parametr K (skala logarytmiczna)"
            plt.title(f"[{prefix.upper()}] Czas wykonania w zależności od parametru {param_col} ({file_name})", fontsize=14, pad=15)
            plt.xlabel(x_label_text, fontsize=12)
            plt.ylabel("Czas wykonania (s)", fontsize=12)
            plt.legend(title="Rozmiar bloku", bbox_to_anchor=(1.05, 1), loc="upper left")
            plt.tight_layout()
            
            output_path_2 = plots_dir_path / f"{prefix}_{filename_clean}_time_vs_{param_col}.png"
            plt.savefig(output_path_2, dpi=300)
            plt.close()
            print(f"Zapisano wykres: {output_path_2}")