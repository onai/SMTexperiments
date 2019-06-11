'''
Rounding
'''

from sklearn.cluster import KMeans
import matplotlib; matplotlib.use('Agg')
import matplotlib.pyplot as pp
import numpy as np

N= int(128000)

N_BINS = 128

N_ROUNDS = 100

AMT = 10 ** 6

def generate_amts():
    return np.random.randint(AMT, size=N) / AMT

def cluster_amts(amts):
    setup = KMeans(n_clusters=N_BINS)
    clusters = setup.fit(amts.reshape(-1,1))
    centers = clusters.cluster_centers_.squeeze()
    plot_cents(centers)
    return clusters.cluster_centers_, clusters.labels_, get_cluster_sums(amts, clusters.labels_)

def get_cluster_sums(amts, labels):
    result = {}
    for i in range(N_BINS):
        cluster_sum = amts[np.where(labels==i)[0]].sum()
        result[i] = (cluster_sum, np.where(labels==i)[0].shape[0])
        
    return result

def plot_cents(centers, round=0):
    pp.clf()
    pp.plot(centers, np.zeros_like(centers), '|')
    fig = pp.gcf()
    fig.set_size_inches(18.5, 10.5)
    pp.savefig('centers' + str(round) + '.png', dpi=100)

def new_round(amts, clusters):
    '''
    choose some existing amts, reduce their amt (just halve for now), 
    introduce new amts with the remainder
    '''

    # keep it equal for now
    n_picked = 100

    picked_amts = np.random.randint(0, len(amts), size=n_picked)
    for amt_idx in picked_amts:
        new_amts, new_clusters = nuke_old_amt(amts, amt_idx, clusters)
        #new_amt = amts[amt_idx] - new_amts[amt_idx]
        new_amt = np.random.randint(AMT, AMT * 100, size=1).squeeze()
        amts, clusters = add_new_amt(new_amts, new_clusters, new_amt)

    return amts, clusters

def nuke_old_amt(amts, amt_idx, clusters):
    '''
    '''
    old_amt = amts[amt_idx]
    new_amt = old_amt // 2

    # get cluster id, update this cluster's center
    cur_cluster = clusters[1][amt_idx]
    cluster_center = clusters[0][cur_cluster]

    new_cluster = get_new_cluster(new_amt, clusters[0])

    cluster_attrs = clusters[-1]
    cur_sum, cur_size = cluster_attrs[cur_cluster]
    new_sum = cur_sum - old_amt
    new_size = cur_size - 1

    cluster_attrs[cur_cluster] = (new_sum, new_size)
    amts[amt_idx] = new_amt

    new_center = new_sum / new_size
    clusters[0][cur_cluster] = new_center
    clusters[1][amt_idx] = new_cluster

    return amts, (clusters[0], clusters[1], cluster_attrs)

def add_new_amt(amts, clusters, new_amt):
    new_cluster = get_new_cluster(new_amt, clusters[0])

    new_amts = np.append(amts, new_amt)
    new_labels = np.append(clusters[1], new_cluster)
    
    cluster_attrs = clusters[-1]
    new_size = cluster_attrs[new_cluster][1] + 1
    new_sum = cluster_attrs[new_cluster][0] + new_amt

    cluster_attrs[new_cluster] = (new_sum, new_size)
    new_center = new_sum / new_size

    clusters[0][new_cluster] = new_center
    new_clusters = np.append(clusters[1], new_cluster)

    return new_amts, (clusters[0], new_clusters, cluster_attrs)

def get_new_cluster(new_amt, centers):
    min_dist = AMT * 100
    cl_id = -1
    for i, center in enumerate(centers):
        if abs(center - new_amt) < min_dist:
            min_dist = abs(center - new_amt)
            cl_id = i

    return cl_id

def metric(amts, clusters):
    cluster_amts = np.zeros_like(amts)
    denominators = np.zeros_like(amts)
    for i, cluster_id in enumerate(clusters[1]):
        cluster_amts[i] = clusters[0][cluster_id]
        denominators[i] = clusters[-1][cluster_id][0]

    fractions = amts / amts.sum()
    approx_fractions = cluster_amts / cluster_amts.sum()

    diff = fractions - approx_fractions
    return np.square(diff).sum(), diff


if __name__ == '__main__':
    amts = generate_amts()

    centers, labels, attrs = cluster_amts(amts)

    for i in range(N_ROUNDS):
        score, dist = metric(amts, (centers, labels, attrs))
        print(i, score)
        print(np.histogram(dist))
        print(np.histogram(amts))
        amts, (centers, labels, attrs) = new_round(amts, (centers, labels, attrs))
        
