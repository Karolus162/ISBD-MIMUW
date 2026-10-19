import csv
import subprocess
from pathlib import Path

files_dir_path = Path("files")
files_dir_path.mkdir(exist_ok=True)

bench_files = {
    "file_4kb": {
        "size": 4 * 1024,
        "blocks": [8, 32, 128, 512, 4 * 1024]
    },
    "file_4mb": {
        "size": 4 * 1024 * 1024,
        "blocks": [512, 4 * 1024, 64 * 1024, 1024 * 1024, 4 * 1024 * 1024]
    }
}

for filename, config in bench_files.items():
    file_path = files_dir_path / filename
    file_size = config["size"]
    
    if not file_path.exists():
        with open(file_path, "wb") as f:
            f.seek(file_size - 1)
            f.write(b"\0")

bench_threads = [1, 2, 4, 8, 16]
iterations = 5

latency_benchmark_results = "latency_benchmark_results.csv"
results = []

for filename, config in bench_files.items():
    file_size = config["size"]
    file_block_sizes = config["blocks"]
    
    for block_size in file_block_sizes:
        for threads in bench_threads:
            file_path = files_dir_path / filename
            cmd = ["cargo", "run", "--release", "--bin", "latency", str(file_path), str(block_size), str(threads)]
            
            print(f"Testowanie latency: file={filename}, block_size={block_size}, threads={threads}")
            
            measured_times = []
            
            for _ in range(iterations):
                subprocess.run(
                    "sudo sysctl -w vm.drop_caches=3", 
                    shell=True, 
                    check=True, 
                    stdout=subprocess.DEVNULL, 
                    stderr=subprocess.DEVNULL
                )
                result = subprocess.run(
                    cmd, 
                    capture_output=True, 
                    text=True, 
                    check=True
                )
                
                output = result.stdout
                time_micros = int(result.stdout.strip())
                
                measured_times.append(time_micros)
            
            min_time_micros = min(measured_times)

            results.append({
                "file": filename,
                "block_size": block_size,
                "threads": threads,
                "time_seconds": min_time_micros / 1_000_000
            })

with open(latency_benchmark_results, mode="w", newline="", encoding="utf-8") as f:
    writer = csv.DictWriter(f, fieldnames=["file", "block_size", "threads", "time_seconds"])
    writer.writeheader()
    writer.writerows(results)


throughput_benchmark_results = "throughput_benchmark_results.csv"
throughput_results = []

for filename, config in bench_files.items():
    file_size = config["size"]
    file_block_sizes = config["blocks"]
    
    for block_size in file_block_sizes:
        file_path = files_dir_path / filename
        cmd = ["cargo", "run", "--release", "--bin", "throughput", str(file_path), str(block_size)]
        
        print(f"Testowanie throughput : file={filename}, block_size={block_size}")
        
        measured_times = []
        
        for _ in range(iterations):
            subprocess.run(
                "sudo sysctl -w vm.drop_caches=3", 
                shell=True, 
                check=True, 
                stdout=subprocess.DEVNULL, 
                stderr=subprocess.DEVNULL
            )
            result = subprocess.run(
                cmd, 
                capture_output=True, 
                text=True, 
                check=True
            )
            
            output = result.stdout
            time_micros = int(result.stdout.strip())
            
            measured_times.append(time_micros)
        
        min_time_micros = min(measured_times)

        throughput_results.append({
            "file": filename,
            "block_size": block_size,
            "time_seconds": min_time_micros / 1_000_000
        })

with open(throughput_benchmark_results, mode="w", newline="", encoding="utf-8") as f:
    writer = csv.DictWriter(f, fieldnames=["file", "block_size", "time_seconds"])
    writer.writeheader()
    writer.writerows(throughput_results)

through_old_benchmark_results = "through_old_benchmark_results.csv"
through_old_results = []

for filename, config in bench_files.items():
    file_size = config["size"]
    file_block_sizes = config["blocks"]
    
    for block_size in file_block_sizes:
        file_path = files_dir_path / filename
        cmd = ["cargo", "run", "--release", "--bin", "throughput_old", str(file_path), str(block_size)]
        
        print(f"Testowanie throughput_old: file={filename}, block_size={block_size}")
        
        measured_times = []
        
        for _ in range(iterations):
            subprocess.run(
                "sudo sysctl -w vm.drop_caches=3", 
                shell=True, 
                check=True, 
                stdout=subprocess.DEVNULL, 
                stderr=subprocess.DEVNULL
            )
            result = subprocess.run(
                cmd, 
                capture_output=True, 
                text=True, 
                check=True
            )
            
            output = result.stdout
            time_micros = int(result.stdout.strip())
            
            measured_times.append(time_micros)
        
        min_time_micros = min(measured_times)

        through_old_results.append({
            "file": filename,
            "block_size": block_size,
            "time_seconds": min_time_micros / 1_000_000
        })

with open(through_old_benchmark_results, mode="w", newline="", encoding="utf-8") as f:
    writer = csv.DictWriter(f, fieldnames=["file", "block_size", "time_seconds"])
    writer.writeheader()
    writer.writerows(through_old_results)

pipeline_benchmark_results = "pipeline_benchmark_results.csv"
pipeline_results = []

k_values = [1, 10000, 50000]

for filename, config in bench_files.items():
    file_size = config["size"]
    file_block_sizes = config["blocks"][2:]
    
    for block_size in file_block_sizes:
        for k in k_values:
            file_path = files_dir_path / filename
            cmd = ["cargo", "run", "--release", "--bin", "pipeline", str(file_path), str(block_size), str(k)]
            
            print(f"Testowanie pipeline: file={filename}, block_size={block_size}, k={k}")
            
            measured_times = []
            
            for _ in range(iterations):
                subprocess.run(
                    "sudo sysctl -w vm.drop_caches=3", 
                    shell=True, 
                    check=True, 
                    stdout=subprocess.DEVNULL, 
                    stderr=subprocess.DEVNULL
                )
                result = subprocess.run(
                    cmd, 
                    capture_output=True, 
                    text=True, 
                    check=True
                )
                
                output = result.stdout
                time_micros = int(result.stdout.strip())
                
                measured_times.append(time_micros)
            
            min_time_micros = min(measured_times)

            pipeline_results.append({
                "file": filename,
                "block_size": block_size,
                "k": k,
                "time_seconds": min_time_micros / 1_000_000
            })

with open(pipeline_benchmark_results, mode="w", newline="", encoding="utf-8") as f:
    writer = csv.DictWriter(f, fieldnames=["file", "block_size", "k", "time_seconds"])
    writer.writeheader()
    writer.writerows(pipeline_results)

### optymalne k dla M = 1M, dla pliku 64mb
# def find_opt_k():
#     file_path = Path("files") / "file_64mb"
#     file_size = 64 * 1024 * 1024
#     if not file_path.exists():
#         with open(file_path, "wb") as f:
#             f.seek(file_size - 1)
#             f.write(b"\0")
#     block_size = 1024 * 1024
#     cmd = ["cargo", "run", "--release", "--bin", "pipeline", str(file_path), str(block_size), str(1)]
#     measured_times = []
#     for _ in range(5):
#         subprocess.run(
#             "sudo sysctl -w vm.drop_caches=3", 
#             shell=True, 
#             check=True, 
#             stdout=subprocess.DEVNULL, 
#             stderr=subprocess.DEVNULL
#         )
#         result = subprocess.run(
#             cmd, 
#             capture_output=True, 
#             text=True, 
#             check=True
#         )
#         output = result.stdout
#         time_micros = int(result.stdout.strip())
#         measured_times.append(time_micros)
#     first_time = min(measured_times)
#     time = min(measured_times)
#     k = 10000

#     while (time <= first_time * 1,1):
#         k *= 2
#         cmd = ["cargo", "run", "--release", "--bin", "pipeline", str(file_path), str(block_size), str(k)]
#         measured_times = []
#         for _ in range(5):
#             subprocess.run(
#                 "sudo sysctl -w vm.drop_caches=3", 
#                 shell=True, 
#                 check=True, 
#                 stdout=subprocess.DEVNULL, 
#                 stderr=subprocess.DEVNULL
#             )
#             result = subprocess.run(
#                 cmd, 
#                 capture_output=True, 
#                 text=True, 
#                 check=True
#             )
            
#             output = result.stdout
#             time_micros = int(result.stdout.strip())
#             measured_times.append(time_micros)
#         time = min(measured_times)
#         print('xd')
#     print(k/2)

# find_opt_k()