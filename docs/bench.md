Benchmarking keygen t=2 n=3: Warming up for 3.0000 s
Warning: Unable to complete 10 samples in 5.0s. You may wish to increase target time to 17.3s.
keygen t=2 n=3          time:   [337.85 ms 373.00 ms 397.72 ms]                         
                        change: [-97.577% -97.210% -96.863%] (p = 0.00 < 0.05)
                        Performance has improved.

Benchmarking keygen t=2 n=5: Warming up for 3.0000 s
Warning: Unable to complete 10 samples in 5.0s. You may wish to increase target time to 37.5s.
keygen t=2 n=5          time:   [606.88 ms 638.04 ms 678.44 ms]                         
                        change: [-41.503% -36.972% -31.985%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 1 outliers among 10 measurements (10.00%)
  1 (10.00%) high mild

Benchmarking keygen t=4 n=8: Warming up for 3.0000 s
Warning: Unable to complete 10 samples in 5.0s. You may wish to increase target time to 70.5s.
keygen t=4 n=8          time:   [1.1059 s 1.1489 s 1.1709 s]                            
                        change: [-53.356% -49.902% -46.569%] (p = 0.00 < 0.05)
                        Performance has improved.

Benchmarking keygen t=16 n=20: Warming up for 3.0000 s
Warning: Unable to complete 10 samples in 5.0s. You may wish to increase target time to 65.7s.
keygen t=16 n=20        time:   [1.1282 s 1.1579 s 1.2028 s]                              

     Running target/release/deps/gg18_sign-cf1ed97341a94234
Gnuplot not found, using plotters backend
Benchmarking keygen t=2 n=5: Warming up for 3.0000 s
Warning: Unable to complete 10 samples in 5.0s. You may wish to increase target time to 51.2s.
keygen t=2 n=5          time:   [924.31 ms 969.26 ms 1.0060 s]                          
                        change: [+33.032% +44.179% +56.440%] (p = 0.00 < 0.05)
                        Performance has regressed.

Benchmarking keygen t=4 n=8: Warming up for 3.0000 s
Warning: Unable to complete 10 samples in 5.0s. You may wish to increase target time to 109.6s.
keygen t=4 n=8          time:   [2.0903 s 2.1321 s 2.1756 s]                            
                        change: [+78.128% +87.125% +97.552%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 1 outliers among 10 measurements (10.00%)
  1 (10.00%) low mild

Benchmarking keygen t=16 n=20: Warming up for 3.0000 s
Warning: Unable to complete 10 samples in 5.0s. You may wish to increase target time to 682.9s.
keygen t=16 n=20        time:   [11.187 s 11.656 s 12.158 s]                              
                        change: [+855.57% +897.19% +936.37%] (p = 0.00 < 0.05)
                        Performance has regressed.
